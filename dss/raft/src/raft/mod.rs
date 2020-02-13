//! The raft implementation.
//!
//! # Raft(lab 2)
//! ## general
//! Generally, it starts at `transform_to_follower`, and transform states by the
//! `transform_*` family, all of which accepts `Arc<Mutex<Raft>>`.
//!
//! All rpc handlers are blocking(!), so `async_rpc!` macro simply start new thread for handler, incase of
//! them blocks the coroutine runtime.
//!
//! This implementation uses thread model, which will bear more context switch and inter-core sync overhead.
//! But for the coroutine approach, `future 0.1`, which is really tricky for me to adapt it with `async/.await`,
//! I don't think I can do it well, or at least, be happy when facing chaotic type and compile error information.
//! All tests just have run on Windows and macOS... I wish it won't fail on Linux...
//!
//! ## where to find algorithm implementation
//! ### election(2A)
//! election starts from `election_timer` fires, and call to `transform_to_candidate`.
//!
//! handler of `RequestVotes` is `do_request_votes`.
//!
//! ### log replication(2B)
//! leader sending rpc starts from `transform_to_leader`, but most of logic is in `modify_state_by_append_entries`.
//!
//! follower handles rpc starts from `do_append_entries`, but most of logic is in `do_append_entries_judge`.
//!
//! ### persist(2C)
//! Persisting logic is in `persist` and `restore` function.
//!
//! The optimization that needed for passing `unreliable_figure8_2c` logic is in `do_append_entries`(follower site),
//! and `modify_state_by_append_entries`(leader site).
//!
use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::{Index, RangeFrom};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::sync::mpsc::UnboundedSender;
use futures::Future;
use rand::Rng;
use rayon::ThreadPoolBuilder;

use labcodec::{decode, encode};
use labrpc::RpcFuture;

use crate::proto::raftpb::raft::Client;
use crate::proto::raftpb::*;
use crate::raft::RaftRole::{Candidate, Follower, Leader};
use crate::{
    select, ThreadPoolWithDrop, Timer,
    TimerMsg::{self, *},
};

use super::async_rpc;

use self::errors::*;
use self::persister::*;

#[cfg(test)]
pub mod config;
pub mod errors;
pub mod persister;
#[cfg(test)]
mod tests;

/// the snapshot of raft state.
pub struct SnapshotFile {
    pub commands: Vec<Vec<u8>>,
}

impl Default for SnapshotFile {
    fn default() -> Self {
        SnapshotFile { commands: vec![] }
    }
}

/// A vector with offset.
struct RaftLogWithSnapShot {
    last_included_index: u64,
    last_included_term: u64,
    state_machine_state: SnapshotFile,
    commands: Vec<LogEntry>,
}

impl Default for RaftLogWithSnapShot {
    fn default() -> Self {
        RaftLogWithSnapShot {
            last_included_index: 0,
            last_included_term: 0,
            state_machine_state: SnapshotFile::default(),
            commands: vec![],
        }
    }
}

impl Into<RaftLogWithSnapShot> for InstallSnapshotArgs {
    fn into(self) -> RaftLogWithSnapShot {
        RaftLogWithSnapShot {
            last_included_index: self.last_included_index,
            last_included_term: self.last_included_term,
            state_machine_state: SnapshotFile {
                commands: self.data,
            },
            commands: vec![],
        }
    }
}

impl RaftLogWithSnapShot {
    fn is_in_snapshot(&self, index: usize) -> bool {
        self.checked_offset_index(index).is_none()
    }

    fn offset_index(&self, origin: usize) -> usize {
        self.checked_offset_index(origin).unwrap_or_else(||
            panic!("Trying to access a entry that is in the snapshot: snapshot last index = {}, accessing = {}",
                   self.last_included_index,
                   origin, )
        )
    }

    /// Try to get the origin index in the log.
    /// If the index is in snapshot, return `None`.
    fn checked_offset_index(&self, origin: usize) -> Option<usize> {
        origin.checked_sub((self.last_included_index + 1) as usize)
    }

    /// Get length of the logs.
    fn len(&self) -> usize {
        self.last_included_index as usize + self.commands.len()
    }

    /// Push a new log entry into log.
    fn push(&mut self, log: LogEntry) {
        self.commands.push(log)
    }

    /// Get a log entry at the exact place.
    /// like `log[n]`
    fn get(&self, n: usize) -> Option<&LogEntry> {
        if n == 0 {
            return None;
        }
        self.commands.get(self.offset_index(n))
    }

    /// Get all entries after the nth place of log entry.
    /// like `log[n..]`
    fn after(&self, n: usize) -> &[LogEntry] {
        let offset_idx = self.offset_index(n);
        &self.commands[offset_idx..]
    }

    /// Remove all entries after `target_size`.
    /// i.e. shrink the vector length to `target_size`.
    fn truncate(&mut self, target_size: usize) {
        self.commands.truncate(self.offset_index(target_size))
    }

    /// Get the term of the last entry.
    /// When here isn't any entry, returns `0`.
    fn last_term(&self) -> u64 {
        self.commands
            .last()
            .map(|e| e.term)
            .unwrap_or(self.last_included_term)
    }

    fn term_at(&self, n: usize) -> u64 {
        self.checked_offset_index(n)
            .map(|after_offset| self.commands.get(after_offset).map(|e| e.term).unwrap_or(0))
            .unwrap_or(self.last_included_term)
    }

    /// Return the iter of current log vector.
    /// [Excludes the snapshot!]
    fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.commands.iter()
    }
}

impl Index<usize> for RaftLogWithSnapShot {
    type Output = LogEntry;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Index out of bound.")
    }
}

impl Index<RangeFrom<usize>> for RaftLogWithSnapShot {
    type Output = [LogEntry];

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        self.after(index.start)
    }
}

/// Some additional configuration options of Raft.
struct RaftConfig {
    /// Depends how often the leader sends append_entries during idle periods.
    leader_append_entries_delay: Duration,
    /// Tolerance for high latency.
    /// The higher this value, the higher the probability of receiving an append_entries response in a high-latency network.
    /// However, in order to prevent IO from blocking new requests, more threads will be started.
    latency_tolerance_factor: f64,
}

impl RaftConfig {
    fn get_timeout(&self) -> Duration {
        self.leader_append_entries_delay
            .mul_f64(self.latency_tolerance_factor)
    }
}

impl Default for RaftConfig {
    fn default() -> Self {
        RaftConfig {
            leader_append_entries_delay: Duration::from_millis(40),
            latency_tolerance_factor: 1.5,
        }
    }
}

pub struct ApplyMsg {
    pub command_valid: bool,
    pub command: Vec<u8>,
    pub command_index: u64,
}

/// State of a raft peer.
#[derive(Default, Clone, Debug)]
pub struct State {
    pub term: u64,
    pub is_leader: bool,
}

impl State {
    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.term
    }
    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
}

#[allow(dead_code)]
struct SentAppendEntriesRequest {
    follower: usize,
    request: AppendEntriesArgs,
    response: Receiver<Result<AppendEntriesReply>>,
}

struct SentRequest<Req, Res> {
    follower: usize,
    request: Req,
    response: Receiver<Result<Res>>,
}

impl<Arg, Rep> From<(usize, Arg, Receiver<Result<Rep>>)> for SentRequest<Arg, Rep> {
    fn from(origin: (usize, Arg, Receiver<Result<Rep>>)) -> Self {
        SentRequest {
            follower: origin.0,
            request: origin.1,
            response: origin.2,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Copy, Debug)]
enum RaftRole {
    Leader = 0,
    Candidate = 1,
    Follower = 2,
}

// A single Raft peer.
pub struct Raft {
    // RPC end points of all peers
    peers: Vec<RaftClient>,
    // Object to hold this peer's persisted state
    persister: Box<dyn Persister>,
    // this peer's index into peers[]
    me: usize,
    state: Arc<State>,

    // state a Raft server must maintain.
    apply_ch: UnboundedSender<ApplyMsg>,
    current_role: RaftRole,
    election_timer: Option<Timer>,

    // stored state.
    term: u64,
    voted_for: Option<usize>,
    // TODO: Generify Log by `RaftLog` trait.
    log: RaftLogWithSnapShot,

    // in-memory state
    commit_index: u64,
    last_applied: u64,

    // leader state
    leader_state: Option<LeaderState>,
    /// the thread pool to execute leader rpc.
    leader_execution_pool: ThreadPoolWithDrop,

    // misc
    /// config, including timeout and thread pool size.
    extra: RaftConfig,
    /// the byte size of log of last snapshot.
    log_size: usize,
}

/// A raft log entry.
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub data: Vec<u8>,
    pub term: u64,
}

impl LogEntry {
    fn new(data: Vec<u8>, term: u64) -> Self {
        LogEntry { data, term }
    }
}

#[derive(Debug, Clone)]
struct LeaderState {
    next_index: Vec<u64>,
    match_index: Vec<u64>,
    new_request: Sender<()>,
}

impl LeaderState {
    fn by_raft(raft: &Raft, sx: Sender<()>) -> Self {
        LeaderState {
            next_index: vec![raft.last_log_index() + 1; raft.peers.len()],
            match_index: vec![0; raft.peers.len()],
            new_request: sx,
        }
    }
}

impl PersistedStatus {
    fn by_raft(raft: &Raft) -> Self {
        // We can trivially persist all log entries(which is also Figure 2 tell us to do),
        // but this in partition scenario will exceed the log size limit sometimes
        // (about 10% of `snapshot_unreliable_recover_concurrent_partition_linearizable` tests).
        //
        // ... so I tried to optimize them.
        // ... my journey begins at just persisting committed logs.
        // ... which is buggy.
        // A sample scenario:
        // Firstly, out leader commit a entry at index X by counting replicate.
        // ... and apply it.
        // ... ...(This is acceptable, dut to leader completeness property.)
        // ... and then Leader died.
        // ... and then one of the follower became Leader.
        // ... ...(This is possible, because we only persist committed log.)
        // ... and new Leader recorded new entry at X.
        // ... and this would finally overwrite original committed log entry at X.
        // ... and we would see a error probably, because the leader completeness property (at Figure 3) broken.
        // FOLLOWER MUST KEEP ALL LOG ENTRIES!
        // And it's harder to trigger this bug on harder test...
        // ... (This conclusion is made by just experience:
        // ... 100 tests of snapshot_unreliable_recover_concurrent_partition_linearizable doesn't
        // ... fail even leaving this bug unfixed.)
        //
        // Finally, what did I do?
        // Just take a snapshot at `kvraft::server::Node::kill`,
        // which is tricky, but maybe effective :).
        let logs = raft.log.iter().cloned().map(Into::into).collect();

        PersistedStatus {
            current_term: raft.term,
            voted_for: raft.voted_for.iter().map(|x| *x as u64).collect(),
            logs,
        }
    }
}

impl Snapshot {
    fn by_raft(raft: &Raft) -> Self {
        Snapshot {
            state_machine_state: raft.log.state_machine_state.commands.clone(),
            last_index_of_snapshot: raft.log.last_included_index,
            last_term_of_snapshot: raft.log.last_included_term,
        }
    }
}

impl Raft {
    /// the service or tester wants to create a Raft server. the ports
    /// of all the Raft servers (including this one) are in peers. this
    /// server's port is peers[me]. all the servers' peers arrays
    /// have the same order. persister is a place for this server to
    /// save its persistent state, and also initially holds the most
    /// recent saved state, if any. apply_ch is a channel on which the
    /// tester or service expects Raft to send ApplyMsg messages.
    /// This method must return quickly.
    pub fn new(
        peers: Vec<RaftClient>,
        me: usize,
        persister: Box<dyn Persister>,
        apply_ch: UnboundedSender<ApplyMsg>,
    ) -> Raft {
        let raft_state = persister.raft_state();
        let snapshot = persister.snapshot();
        // Your initialization code here (2A, 2B, 2C).
        let peer_count = peers.len();
        let config = RaftConfig::default();
        let mut rf = Raft {
            peers,
            persister,
            me,
            state: Arc::default(),
            apply_ch,
            current_role: Follower,
            term: 0,
            voted_for: None,
            election_timer: None,
            log: RaftLogWithSnapShot::default(),
            commit_index: 0,
            last_applied: 0,
            leader_state: None,
            leader_execution_pool: ThreadPoolBuilder::new()
                .thread_name(move |n| format!("[`{}`] leader execution worker ({})", me, n))
                .num_threads(((peer_count as f64) * config.latency_tolerance_factor) as usize)
                .build()
                .unwrap()
                .into(),
            extra: config,
            log_size: 0,
        };

        // initialize from state persisted before a crash
        rf.restore(&raft_state, &snapshot);

        rf
    }

    /// save Raft's persistent state to stable storage,
    /// where it can later be retrieved after a crash and restart.
    /// see paper's Figure 2 for a description of what should be persistent.
    fn persist(&mut self) {
        let persisted = PersistedStatus::by_raft(self);
        let snapshot = Snapshot::by_raft(self);
        let mut log_buf = vec![];
        encode(&persisted, &mut log_buf).unwrap();
        self.log_size = log_buf.len();
        let mut snapshot_buf = vec![];
        encode(&snapshot, &mut snapshot_buf).unwrap();
        self.persister
            .save_state_and_snapshot(log_buf, snapshot_buf);
    }

    /// restore previously persisted state.
    fn restore(&mut self, log: &[u8], snapshot: &[u8]) {
        if log.is_empty() && snapshot.is_empty() {
            info!("{} bootstrap without any state!", self.self_info());
            return;
        }
        self.log_size = log.len();
        decode::<PersistedStatus>(log)
            .and_then(|state| {
                decode::<Snapshot>(snapshot).map(|ss| {
                    self.term = state.current_term;
                    self.log = RaftLogWithSnapShot {
                        last_included_index: ss.last_index_of_snapshot,
                        last_included_term: ss.last_term_of_snapshot,
                        state_machine_state: SnapshotFile {
                            commands: ss.state_machine_state,
                        },
                        commands: state.logs.into_iter().map(Into::into).collect(),
                    };
                    self.voted_for = state.voted_for.first().map(|x| *x as usize);
                    // let apply the snapshots message to state machine firstly...
                    // We can assert that snapshot are committed.
                    self.commit_index = self.log.last_included_index;
                    self.apply_snapshot();
                    info!(
                        "{} bootstrap with log length {}!",
                        self.self_info(),
                        self.log.len()
                    )
                })
            })
            .expect("failed to decode persisted status.");
    }

    /// apply snapshot to state machine.
    /// This updates `last_applied`.
    fn apply_snapshot(&mut self) {
        for e in self.log.state_machine_state.commands.iter() {
            let msg = ApplyMsg {
                command_valid: false,
                command: e.clone(),
                command_index: 0,
            };
            if self.apply_ch.unbounded_send(msg).is_err() {
                error!(
                    "{} failed to send snapshot state, which probably cause unexpected behavior.",
                    self.self_info()
                );
            }
        }

        self.last_applied = self.log.last_included_index;
        info!(
            "{} applied (by snapshot) to {}",
            self.self_info(),
            self.last_applied
        );
    }

    /// send a rpc request to a peer.
    ///
    /// # arguments
    /// - server: the rpc endpoint index.
    /// - args: the rpc args.
    /// - rpc: the code segment that uses client and args to send rpc.
    ///
    /// # returns
    /// a `SentRequest` struct.
    fn send_request<Arg, Rep: Send + 'static>(
        &self,
        server: usize,
        args: Arg,
        rpc: impl Fn(&Client, &Arg) -> RpcFuture<Rep>,
    ) -> SentRequest<Arg, Rep> {
        let peer = &self.peers[server];
        let (tx, rx) = channel::<Result<Rep>>();
        peer.spawn(rpc(peer, &args).map_err(Error::Rpc).then(move |res| {
            let result = tx.send(res);
            if let Err(e) = result {
                debug!("send_request: result of RPC is unused. since: {:?}", e);
            }
            Ok(())
        }));
        SentRequest::from((server, args, rx))
    }

    /// get the current state string of this raft.
    fn self_info(&self) -> String {
        format!(
            "[{} t{} c{} a{} l{} s{} {:?}]",
            self.me,
            self.term,
            self.commit_index,
            self.last_applied,
            self.log.len(),
            self.log.last_included_index,
            self.current_role,
        )
    }

    /// make a new log entry contains `data`.
    fn make_log(&self, data: Vec<u8>) -> LogEntry {
        LogEntry::new(data, self.term)
    }

    /// show log terms of current node.
    fn log_info(&self) -> String {
        format!(
            "({} snapshots omitted.) {:?}",
            self.log.last_included_index,
            self.log.iter().map(|e| e.term).collect::<Vec<_>>()
        )
    }

    /// add a new command to the raft cluster.
    ///
    /// # returns
    /// when success, return the (index, term) 2-tuple of the command.
    /// (this doesn't means that this command will eventually appears at there,
    ///  before committed, this command can be lost.)
    /// if this raft isn't leader, return `NotLeader`.
    fn start<M>(&mut self, command: &M) -> Result<(u64, u64)>
    where
        M: labcodec::Message,
    {
        let is_leader = self.current_role == Leader;
        if !is_leader {
            return Err(Error::NotLeader);
        }

        let mut buf = vec![];
        debug!(
            "{} get command: {:?}(logs = {:?})",
            self.self_info(),
            command,
            self.log_info()
        );
        labcodec::encode(command, &mut buf).map_err(Error::Encode)?;
        let entry = self.make_log(buf);
        self.log.push(entry);

        let index = self.last_log_index();
        let term = self.term;
        let ls = self.leader_state.as_ref().unwrap();
        ls.new_request
            .send(())
            .unwrap_or_else(|_| error!("leader is died when try send to new_request_channel."));
        Ok((index, term))
    }
}

impl Raft {
    /// Only for suppressing deadcode warnings.
    #[doc(hidden)]
    pub fn __suppress_deadcode(&mut self) {
        let _ = self.start(&0);
        let _ = &self.state;
        let _ = &self.me;
        let _ = &self.persister;
        let _ = &self.peers;

        // user added.
    }
}

// Choose concurrency paradigm.
//
// You can either drive the raft state machine by the rpc framework,
//
// ```rust
// struct Node { raft: Arc<Mutex<Raft>> }
// ```
//
// or spawn a new thread runs the raft state machine and communicate via
// a channel.
//
// ```rust
// struct Node { sender: Sender<Msg> }
// ```
#[derive(Clone)]
pub struct Node {
    raft: Arc<Mutex<Raft>>,
}

impl Into<LogEntry> for ProtoEntry {
    fn into(self) -> LogEntry {
        LogEntry::new(self.command, self.term)
    }
}

impl Into<ProtoEntry> for LogEntry {
    fn into(self) -> ProtoEntry {
        ProtoEntry {
            command: self.data,
            term: self.term,
        }
    }
}

fn reverse_order<T: Ord>(a: &T, b: &T) -> Ordering {
    b.cmp(a)
}

/// Find the mid number of the items array.
/// This function assumes that the `items` array contains a 'supreme' element,
/// which is the `match_index` of the current node itself.
fn mid<T: Ord>(items: &mut [T]) -> &T {
    items.sort_by(reverse_order);
    let mid = (items.len() - 1) / 2;
    &items[mid]
}

impl Raft {
    /// check if the current node is leader.
    fn is_leader(&self) -> bool {
        self.current_role == Leader
    }

    /// find a index in the log where contains the first log entry of the
    /// specified term.
    fn get_term_starts_at(&self, term: u64, from: usize) -> usize {
        let mut n = from;
        while self.log.term_at(n) == term && !self.log.is_in_snapshot(n) {
            n -= 1;
        }
        n + 1
    }

    /// calculate the next commit index by counting replicates.
    ///
    /// # panics
    /// if `!self.is_leader()`, which means current node is a probably follower,
    /// follower should never try to commit log.
    fn next_commit_index(&self) -> u64 {
        if !self.is_leader() {
            panic!("fetal: try to fetch leader state on non-leader node");
        }
        let leader_state = self
            .leader_state
            .as_ref()
            .expect("fetal: leader node does'nt have leader state.");
        let mut valid_state = leader_state.match_index.clone();
        valid_state.remove(self.me);
        *mid(valid_state.as_mut_slice())
    }

    /// make a `ApplyMessage` with the log entry at `index`.
    fn make_apply_message(&self, index: u64) -> ApplyMsg {
        let log = &self.log[index as usize];
        ApplyMsg {
            command_valid: true,
            command: log.data.clone(),
            command_index: index,
        }
    }

    /// Apply logs by current `commit_index` to state machine.
    fn apply_logs(&mut self) {
        for i in (self.last_applied + 1)..=(self.commit_index) {
            self.apply_ch
                .unbounded_send(self.make_apply_message(i))
                .unwrap_or_else(|e| error!("{} failed to send to apply ch. because: {}. the client of raft may shutdown.", self.self_info(), e));
        }
        self.last_applied = self.commit_index;
        info!(
            "{} applied to index {}.",
            self.self_info(),
            self.last_applied,
        );
    }

    /// Leader commit its indices by this function.
    /// Comparing to `next_commit_index`, this function will do some extra checking.
    /// For example, this function checks term of last log entry to prevent committing log entries
    /// from last term by counting replicas.
    fn leader_commit_logs(&mut self) {
        let next = self.next_commit_index();
        assert!((next as usize) <= self.log.len(),
                "match_index grater than self log length... next = {} and match_index = {:?} and self.log.len() = {}",
                next, self.leader_state.as_ref().map(|s| s.match_index.clone()), self.log.len());
        // 5.4.2: NEVER commit log entries from previous terms by counting replicas.
        if next > self.commit_index && self.log[next as usize].term == self.term {
            self.commit_index = next;
            self.persist();
            self.apply_logs();
        }
    }

    /// Get the index of last log.
    /// which is the same as log size.
    fn last_log_index(&self) -> u64 {
        self.log.len() as u64
    }

    /// Get the term of last log.
    /// If no log stored, returns `0`.
    fn last_log_term(&self) -> u64 {
        self.log.last_term()
    }

    /// make `RequestVoteArgs` rpc argument by current state of self.
    fn make_request_vote_args(&self) -> RequestVoteArgs {
        RequestVoteArgs {
            term: self.term,
            candidate_id: self.me as u64,
            last_log_index: self.last_log_index(),
            last_log_term: self.last_log_term(),
        }
    }

    /// transform the raft node to candidate.
    fn transform_to_candidate(raft: Arc<Mutex<Self>>) {
        std::thread::spawn(move || {
            // some basic state transform, and save some cloneable information.
            let mut guard = raft.lock().unwrap();
            guard.current_role = Candidate;
            // make the borrow checker happy.
            let old_term = guard.term;
            guard.update_term(old_term + 1);
            let term_at_start = guard.term;
            let me = guard.me;

            // vote for self, then send `RequestVote` RPCs.
            info!(
                "{} started a new election of term {}.",
                guard.self_info(),
                guard.term
            );
            guard.vote_for(me);
            let peer_count = guard.peers.len();
            let send_result = (0..peer_count)
                .filter(|i| *i != me)
                .map(|i| {
                    guard.send_request(i, guard.make_request_vote_args(), |client, arg| {
                        client.request_vote(arg)
                    })
                })
                .map(|req| req.response)
                .collect::<Vec<Receiver<_>>>();
            drop(guard);

            // let's poll on the requests...
            let data_channel = select(send_result.into_iter());
            let mut vote_count = 1;
            while let Ok(result) = data_channel.recv_timeout(Duration::from_millis(300)) {
                if result.is_err() {
                    continue;
                }
                let vote_result = result.unwrap();

                // 自身已然不再是候选人之时……（从投票节点处得知）
                if vote_result.term > term_at_start {
                    Raft::check_term(raft.clone(), vote_result.term);
                    return;
                }

                vote_count += if vote_result.vote_granted { 1 } else { 0 };

                // Bingo! we get enough votes.
                if vote_count > peer_count / 2 {
                    info!("{} has enough votes at term {}!", me, term_at_start);
                    break;
                }
            }

            // 做一些登基前的准备工作，同时测试自身是否已经被弹劾了。
            let guard = raft.lock().unwrap();
            if vote_count > peer_count / 2
                // ensure that we didn't start another term of election...
                && guard.term == term_at_start
                // ensure that there isn't a leader...
                && guard.current_role == Candidate
            {
                guard.stop_election_timer();
                info!(
                    "{} is now the leader of term {}.",
                    guard.self_info(),
                    term_at_start
                );
                drop(guard);
                Raft::transform_to_leader(raft);
            }
        });
    }

    /// modify leader state by a response of `AppendEntries`.
    ///
    /// # panics
    /// if `!self.is_leader()`.
    fn modify_state_by_append_entries(
        &mut self,
        request: &AppendEntriesArgs,
        response: &AppendEntriesReply,
        follower: usize,
    ) {
        // Don't handle response from older request.
        if self.term != request.term {
            return;
        }

        let self_info = self.self_info();
        let leader_state = self.leader_state.as_mut().unwrap_or_else(|| {
            panic!(
                "{} fetal: Handling append_entries without leader_state.",
                self_info
            )
        });
        if response.success {
            let matching = request.prev_log_index + request.entries.len() as u64;
            // 防止返回乱序……
            let new_match_index = Ord::max(matching, leader_state.match_index[follower]);
            leader_state.match_index[follower] = new_match_index;
            leader_state.next_index[follower] = new_match_index + 1;
            self.leader_commit_logs();
        } else {
            let next_index = response.conflicted_term_starts_at as usize;
            if next_index == 0xcafe_babe {
                panic!("A debug magic number appears, which might means InvalidLeader message has handled by incorrect way.\n\
                Debug info: ({:?}) => {:?} self = {}", request, response, self_info);
            }
            // does follower matches at `conflicted_term`?
            // we have log matching property, don't worry about send from this directly.
            let can_match = self.log.term_at(next_index) == response.conflicted_term;
            let real_next = if can_match {
                next_index
            } else {
                next_index - 1
            } as u64;
            // don't send placeholder!
            leader_state.next_index[follower] = Ord::max(1, real_next);
        }
    }

    /// leader `AppendEntries` response handler.
    /// This do some basic state transform, and check term, authorship,
    /// then delegate tasks to `modify_state_by_append_entries`.
    fn handle_append_entries(
        raft_lock: Arc<Mutex<Self>>,
        request: &AppendEntriesArgs,
        response: &AppendEntriesReply,
        follower: usize,
    ) {
        let mut raft = raft_lock.lock().unwrap();
        let raft_info = raft.self_info();
        debug!(
            "[{}] => {} : append_entries({:?}) => {:?}",
            raft_info, follower, request, response
        );
        if !raft.is_leader() {
            return;
        }
        if raft.term < response.term {
            drop(raft);
            Raft::check_term(raft_lock.clone(), response.term);
            return;
        }
        raft.modify_state_by_append_entries(request, response, follower);
    }

    /// make `AppendEntriesArgs` by current state and target follower.
    fn make_append_entries_for(&self, server: usize) -> AppendEntriesArgs {
        let leader_state = self
            .leader_state
            .as_ref()
            .expect("fetal: try to issue AppendEntries from non-leader node.");
        let next_index = leader_state.next_index[server] as usize;
        AppendEntriesArgs {
            term: self.term,
            leader_id: self.me as u64,
            prev_log_index: (next_index - 1) as u64,
            prev_log_term: self.log.term_at(next_index - 1),
            entries: self.log[next_index..]
                .iter()
                .cloned()
                .map(Into::into)
                .collect(),
            leader_commit: self.commit_index,
        }
    }

    /// check whether a follower is so far behind, that needs
    /// leader sending a `InstallSnapshot` rpc to sync with.
    fn need_install_snapshot(&self, server: usize) -> bool {
        if !self.is_leader() {
            panic!("Calling need_install_snapshot in a non-leader node.")
        }
        let ls = self
            .leader_state
            .as_ref()
            .expect("fetal: leader node without leader state.");
        self.log.is_in_snapshot(ls.next_index[server] as usize)
    }

    /// make `InstallSnapshotArgs` by current state.
    fn make_install_snapshot_args(&self) -> InstallSnapshotArgs {
        InstallSnapshotArgs {
            term: self.term,
            leader_id: self.me as u64,
            last_included_term: self.log.last_included_term,
            last_included_index: self.log.last_included_index,
            data: self.log.state_machine_state.commands.clone(),
        }
    }

    /// transform to Leader。
    fn transform_to_leader(raft_lock: Arc<Mutex<Raft>>) {
        use std::sync::mpsc::RecvTimeoutError;

        // some basic state creation.
        let mut guard = raft_lock.lock().unwrap();
        guard.current_role = Leader;
        let delay = guard.extra.leader_append_entries_delay;
        let (sx, rx) = channel();
        guard.try_send_to_election_timer(Stop);
        guard.leader_state = Some(LeaderState::by_raft(guard.deref(), sx));
        drop(guard);

        // the loop of leader lifetime.
        // when `start` called, the `rx` channel will fire a `()` message.
        // then... we make `AppendEntries` or `InstallSnapshot` to distribute it.
        // or... timeout, we send heartbeat to ensure authorization of our leader.
        while let Err(RecvTimeoutError::Timeout) | Ok(()) = rx.recv_timeout(delay) {
            let raft = raft_lock.lock().unwrap();
            // leader is died.
            if !raft.is_leader() {
                break;
            }
            // after every turn send `AppendEntries`, try to commit logs.
            let raft_info = raft.self_info();
            debug!(
                "{}: leader_state = {:?} (log len = {})",
                raft_info,
                raft.leader_state,
                raft.log.len()
            );

            // NOTE: we can do this by some way more functional...
            let mut append_entries_reqs = vec![];
            let mut install_snapshot_reqs = vec![];
            // send requests.
            for i in 0..raft.peers.len() {
                if i != raft.me {
                    if raft.need_install_snapshot(i) {
                        install_snapshot_reqs.push(raft.send_request(
                            i,
                            raft.make_install_snapshot_args(),
                            |client, args| client.install_snapshot(args),
                        ));
                    } else {
                        append_entries_reqs.push(raft.send_request(
                            i,
                            raft.make_append_entries_for(i),
                            |client, args| client.append_entries(args),
                        ))
                    }
                }
            }
            drop(raft);

            // handle responses.
            Raft::spawn_handler(
                raft_lock.clone(),
                append_entries_reqs.into_iter(),
                Raft::handle_append_entries,
            );
            Raft::spawn_handler(
                raft_lock.clone(),
                install_snapshot_reqs.into_iter(),
                Raft::handle_install_snapshot,
            );
        }
        let mut guard = raft_lock.lock().unwrap();
        guard.leader_state = None;
    }

    /// leader `InstallSnapshot` reply handler.
    fn handle_install_snapshot(
        raft_lock: Arc<Mutex<Raft>>,
        _req: &InstallSnapshotArgs,
        res: &InstallSnapshotReply,
        follower: usize,
    ) {
        let mut raft = raft_lock.lock().unwrap();
        if !raft.is_leader() {
            return;
        }
        if raft.term < res.term {
            drop(raft);
            Raft::check_term(raft_lock.clone(), res.term);
            return;
        }

        let next_idx = raft.log.last_included_index + 1;
        let ls = raft.leader_state.as_mut().unwrap();
        ls.next_index[follower] = next_idx;
        raft.leader_commit_logs()
    }

    /// spawn a handler of some request.
    /// this function uses for the common pattern of sending rpc:
    /// timeout, error-handling, thread spawning.
    fn spawn_handler<Req: Debug + Send + 'static, Res: Send + 'static>(
        raft_lock: Arc<Mutex<Self>>,
        requests: impl Iterator<Item = SentRequest<Req, Res>>,
        handler: impl Fn(Arc<Mutex<Raft>>, &Req, &Res, usize) + Send + Sync + 'static,
    ) {
        let raft = raft_lock.lock().unwrap();
        let h = Arc::new(handler);
        requests.for_each(|req| {
            let raft_info = raft.self_info();
            let raft_lock = raft_lock.clone();
            let delay = raft.extra.get_timeout();
            raft.leader_execution_pool.spawn({
                let raft_lock = raft_lock.clone();
                let h = h.clone();
                move || {
                    match req.response.recv_timeout(delay) {
                        Err(_e) => debug!(
                            "{} failed to receive append_entries result to NO{} because timeout.",
                            raft_info, req.follower
                        ),
                        Ok(Err(e)) => {
                            debug!(
                                "{} Failed to get result of {:?}, because: {}",
                                raft_info, req.request, e
                            );
                        }
                        Ok(Ok(info)) => {
                            h(raft_lock.clone(), &req.request, &info, req.follower);
                        }
                    };
                }
            });
        });
    }

    /// try send a message to election timer.
    ///
    /// # return
    ///
    /// if success to send, return `true`.
    /// otherwise, `false`.
    fn try_send_to_election_timer(&self, message: TimerMsg) -> bool {
        self.election_timer
            .as_ref()
            .map(|sx| sx.sx.send(message))
            .map(|result| result.map(|_| true).unwrap_or(false))
            .unwrap_or(false)
    }

    /// like `try_send_to_election_timer`, but unchecked.
    /// when failed, a warn message will fire.
    fn send_to_election_timer(&self, message: TimerMsg) {
        if !self.try_send_to_election_timer(message) {
            warn!(
                "NO{} send_to_election_timer({:?}): failed try. if this is acceptable, \
                 use try_send_to_election_timer instead.",
                self.me, message
            );
        }
    }

    /// reset the election timer.
    /// when receiving `AppendEntries` or `InstallSnapshot`
    /// from valid leader, call this.
    fn reset_election_timer(&self) {
        if self.is_leader() {
            warn!(
                "NO{} Trying to reset election timer on a leader node.",
                self.me
            );
        }

        self.send_to_election_timer(TimerMsg::Reset);
    }

    /// stop the election timer.
    /// when trans to leader, call this.
    fn stop_election_timer(&self) {
        self.send_to_election_timer(TimerMsg::Stop);
    }

    /// generate the next election timeout.
    fn generate_election_timeout() -> Duration {
        let range = rand::thread_rng().gen_range(150, 300);
        Duration::from_millis(range)
    }

    /// update self.term.
    /// instead of set self.term directly, this function makes sure that
    /// term will increasing monotonically, and do works like persisting.
    ///
    /// # panics
    /// if the new term not greater than current term.
    fn update_term(&mut self, new_term: u64) {
        assert!(
            self.term <= new_term,
            "NO{}(currentTerm = {}) is set to a lower term({}), which is probably an error.",
            self.me,
            self.term,
            new_term
        );
        if new_term != self.term {
            info!("{} is now set to term {}", self.self_info(), new_term);
            self.voted_for = None;
            self.persist();
        }
        self.term = new_term;
    }

    /// transform raft state to follower.
    fn transform_to_follower(raft: Arc<Mutex<Raft>>) {
        let mut guard = raft.lock().unwrap();
        guard.try_send_to_election_timer(Stop);
        guard.current_role = Follower;
        guard.election_timer = Some(Timer::new(Raft::generate_election_timeout, {
            let raft = raft.clone();
            move || Raft::transform_to_candidate(raft.clone())
        }));
    }

    /// check whether self should grant vote to candidate
    /// that issues this `RequestVoteArgs`.
    fn check_grant(&mut self, args: &RequestVoteArgs) -> bool {
        if args.term < self.term {
            return false;
        }
        let self_can_vote =
            self.voted_for.is_none() || self.voted_for == Some(args.candidate_id as usize);
        let self_should_vote = (args.last_log_term > self.last_log_term())
            || (args.last_log_term == self.last_log_term()
                && args.last_log_index >= self.last_log_index());
        self_can_vote && self_should_vote
    }

    /// check whether current term is out-dated.
    /// If it thinks current node is out-dated, transform current node to follower.
    ///
    /// # returns
    /// returns `true` if the raft peer become follower since this function.
    fn check_term(raft: Arc<Mutex<Raft>>, new_term: u64) -> bool {
        let mut guard = raft.lock().unwrap();
        if new_term >= guard.term {
            let old_term = guard.term;
            let leader_to_follower = guard.is_leader() && new_term > old_term;
            let candidate_to_follower = guard.current_role == Candidate;
            guard.update_term(new_term);
            if leader_to_follower || candidate_to_follower {
                info!(
                    "{}, get RPC(term = {}), he eventually has known, he is a follower now.",
                    guard.self_info(),
                    new_term
                );
                drop(guard);
                Raft::transform_to_follower(raft.clone());
                return true;
            }
        }
        false
    }

    /// diff the log with remote log (probably get by `AppendEntries` from leader)
    /// matches the current log.
    ///
    /// # arguments
    /// - base is the start point of remote log, the checking to local log will start at this point.
    /// - entries is the remote log.
    ///
    /// # returns
    /// the length that matches.
    ///
    /// # example
    /// let `self.log   = [1,1,2,3,3]`,
    /// and `remote log = [1,2,4]`,
    /// `check_and_trunc_log(2, [1,2,4])` returns `2` (for `[1,2]` of remote log matches).
    /// and leaving `self.log = [1,1,2]` (truncate any log entries that doesn't matches).
    /// (Raft log starts at index 1, this function, along with `RaftLogWithSnapshot`, follows this.)
    fn check_and_trunc_log(&mut self, base: usize, entries: &[LogEntry]) -> usize {
        for (offset, remote) in entries.iter().enumerate() {
            if self.log.term_at(base + offset) != remote.term {
                self.log.truncate(base + offset);
                return offset;
            }
        }
        entries.len()
    }

    /// vote for the candidate.
    /// comparing to set `self.voted_for` directly, this function checks current vote state,
    /// and persist current state.
    ///
    /// # panics
    /// if the current node has voted for someone.
    fn vote_for(&mut self, candidate: usize) {
        let voted = self.voted_for.map(|x| candidate != x).unwrap_or(false);
        if voted {
            panic!(
                "{} trying to vote {}, but it has voted for {}",
                self.self_info(),
                candidate,
                self.voted_for.unwrap()
            )
        }
        self.voted_for = Some(candidate);
        self.persist();
    }
}
enum FailedAppendEntries {
    ConflictedEntry {
        conflicted_term: u64,
        conflicted_term_starts_at: u64,
    },
    InvalidLeader,
}

impl Node {
    /// Get raft log entries between `start` and `end` (inclusive).
    ///
    /// # returns
    /// The cloned log range.
    ///
    /// # panics
    /// If try to access log that is in snapshot or not committed.
    pub fn log_between(&self, start: usize, end: usize) -> Vec<ApplyMsg> {
        let rf = self.raft.lock().unwrap();
        assert!(
            end <= rf.commit_index as usize,
            "Try to get log entry that not committed."
        );
        (start..=end)
            .map(|i| rf.make_apply_message(i as u64))
            .collect()
    }

    /// Like `log_between`, but returns empty vector when illegal access.
    pub fn try_get_log_between(&self, start: usize, end: usize) -> Vec<ApplyMsg> {
        let rf = self.raft.lock().unwrap();
        if rf.log.is_in_snapshot(start) || end > rf.commit_index as usize {
            return vec![];
        }
        drop(rf);
        self.log_between(start, end)
    }

    /// take the snapshot of `state`, with `last_included_index = last_index`.
    pub fn take_snapshot(&self, state: SnapshotFile, last_index: usize) {
        let mut raft = self.raft.lock().unwrap();
        if raft.log.is_in_snapshot(last_index) {
            error!(
                "{} :( (till_index = {}; last_contains_index = {})",
                raft.self_info(),
                last_index,
                raft.log.last_included_index,
            );
            return;
        }

        assert!(
            last_index <= raft.last_applied as usize,
            "{} Try to take snapshot when not applied!",
            raft.self_info(),
        );

        if raft.is_leader() {
            info!("{} ls = {:?}", raft.self_info(), raft.leader_state);
        }

        let remained_log_starts = raft.log.offset_index(last_index) + 1;
        let new_commands = if remained_log_starts < raft.log.commands.len() {
            raft.log.commands.drain(remained_log_starts..).collect()
        } else {
            vec![]
        };
        let new_log = RaftLogWithSnapShot {
            last_included_index: last_index as u64,
            last_included_term: raft.log.term_at(last_index),
            state_machine_state: state,
            commands: new_commands,
        };
        raft.log = new_log;
        info!(
            "{} takes snapshot (from index: {}), remained log size = {}",
            raft.self_info(),
            last_index,
            raft.log.commands.len(),
        );
        raft.persist();
    }

    /// get current raft log size.
    pub fn log_size(&self) -> usize {
        let raft = self.raft.lock().unwrap();
        raft.log_size
    }

    /// Create a new raft service.
    pub fn new(raft: Raft) -> Node {
        // Your code here.
        let me = raft.me;
        info!("new node NO「{}」started.", me);
        let raft = Arc::new(Mutex::new(raft));
        Raft::transform_to_follower(raft.clone());
        Node { raft: raft.clone() }
    }

    /// the service using Raft (e.g. a k/v server) wants to start
    /// agreement on the next command to be appended to Raft's log. if this
    /// server isn't the leader, returns [Error::NotLeader]. otherwise start
    /// the agreement and return immediately. there is no guarantee that this
    /// command will ever be committed to the Raft log, since the leader
    /// may fail or lose an election. even if the Raft instance has been killed,
    /// this function should return gracefully.
    ///
    /// the first value of the tuple is the index that the command will appear
    /// at if it's ever committed. the second is the current term.
    ///
    /// This method must return without blocking on the raft.
    pub fn start<M>(&self, command: &M) -> Result<(u64, u64)>
    where
        M: labcodec::Message,
    {
        // Your code here.
        // Example:
        // self.raft.start(command)
        let mut raft = self.raft.lock().unwrap();
        raft.start(command)
    }

    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        // Your code here.
        // Example:
        // self.raft.term
        let guard = self.raft.lock().unwrap();
        guard.term
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        // Your code here.
        // Example:
        // self.raft.leader_id == self.id
        let guard = self.raft.lock().unwrap();
        guard.is_leader()
    }

    /// The current state of this peer.
    pub fn get_state(&self) -> State {
        State {
            term: self.term(),
            is_leader: self.is_leader(),
        }
    }

    /// get the current `commit_index` of raft.
    pub fn commit_index(&self) -> u64 {
        let rf = self.raft.lock().unwrap();
        rf.commit_index
    }

    /// reset the election timer of raft.
    pub fn reset_timer(&self) {
        let guard = self.raft.lock().unwrap();
        guard.reset_election_timer();
    }

    /// the tester calls kill() when a Raft instance won't be
    /// needed again. you are not required to do anything in
    /// kill(), but it might be convenient to (for example)
    /// turn off debug output from this instance.
    /// In Raft paper, a server crash is a PHYSICAL crash,
    /// A.K.A all resources are reset. But we are simulating
    /// a VIRTUAL crash in tester, so take care of background
    /// threads you generated with this Raft Node.
    pub fn kill(&self) {
        let mut guard = self.raft.lock().unwrap();
        // stop leader timer.
        guard.current_role = Follower;
        // stop follower timer.
        guard.try_send_to_election_timer(Stop);
        unsafe {
            guard.leader_execution_pool.terminate();
        }
        info!("NO{} is dead.", guard.me);
    }

    /// wrapper for `Raft::check_term`.
    fn check_term(&self, new_term: u64) -> bool {
        Raft::check_term(self.raft.clone(), new_term)
    }

    /// The implementation of AppendEntries.
    /// See the raft paper figure 2.
    /// This function will be executed in an OS thread, don't worry even it blocks.
    fn do_append_entries_judge(
        &self,
        mut args: AppendEntriesArgs,
    ) -> std::result::Result<(), FailedAppendEntries> {
        // pre-handle: check term.
        self.check_term(args.term);

        // 1. Reply false if term < currentTerm.
        if args.term < self.term() {
            return Err(FailedAppendEntries::InvalidLeader);
        }

        // this message is sent by a valid leader, reset election timer.
        self.reset_timer();

        let mut raft = self.raft.lock().unwrap();

        // 2. Reply false if log doesn't match.
        let prev_log_index = args.prev_log_index as usize;
        let term_matches = raft.log.term_at(prev_log_index) == args.prev_log_term;
        if !term_matches {
            if raft.log.is_in_snapshot(prev_log_index) {
                return Err(FailedAppendEntries::ConflictedEntry {
                    conflicted_term: 0,
                    // just let leader reset our index, and send a snapshot.
                    // set to 2 because log[2].term shall never be 0, so leader will set next = 1.
                    conflicted_term_starts_at: 2,
                });
            }

            let entry = raft.log.get(prev_log_index);
            if entry.is_none() {
                return Err(FailedAppendEntries::ConflictedEntry {
                    conflicted_term: 0,
                    // roll back to last index.
                    conflicted_term_starts_at: raft.last_log_index() + 1,
                });
            }

            let conflicted_term = entry.unwrap().term;
            let conflicted_term_starts_at =
                raft.get_term_starts_at(conflicted_term, prev_log_index) as u64;
            return Err(FailedAppendEntries::ConflictedEntry {
                conflicted_term,
                conflicted_term_starts_at,
            });
        }

        // 3. Test matching. If conflict, truncate the log.
        let base = prev_log_index + 1;
        let mut entries: Vec<LogEntry> = args.entries.drain(..).map(Into::into).collect();
        let new_log_base = raft.check_and_trunc_log(base, &entries);

        // 4. Append any new entries.
        let new_logs: Vec<LogEntry> = entries.drain(new_log_base..).collect();
        let log_changed = !new_logs.is_empty();
        for entry in new_logs.into_iter() {
            raft.log.push(entry)
        }

        // 5. Set commit index.
        if args.leader_commit > raft.commit_index {
            let next = Ord::min(args.leader_commit, raft.last_log_index());
            raft.commit_index = next;
            raft.apply_logs();
        }

        // Anyway, persist it.
        if log_changed {
            raft.persist();
        }

        Ok(())
    }

    /// follower handler for `AppendEntries`.
    fn do_append_entries(&self, args: AppendEntriesArgs) -> AppendEntriesReply {
        let success = self.do_append_entries_judge(args);
        match success {
            Ok(()) => AppendEntriesReply {
                term: self.term(),
                success: true,
                conflicted_term: 0,
                conflicted_term_starts_at: 0,
            },
            Err(FailedAppendEntries::InvalidLeader) => AppendEntriesReply {
                term: self.term(),
                success: false,
                // for debug usage -- those fields shouldn't be used.
                // TODO: 使用 oneof 而不是（不太安全的）积类型来完成这项工作。
                conflicted_term: 0xcafe_babe,
                conflicted_term_starts_at: 0xcafe_babe,
            },
            Err(FailedAppendEntries::ConflictedEntry {
                conflicted_term,
                conflicted_term_starts_at,
            }) => AppendEntriesReply {
                term: self.term(),
                success: false,
                conflicted_term,
                conflicted_term_starts_at,
            },
        }
    }

    /// follower handler for `RequestVote`.
    fn do_request_vote(&self, args: RequestVoteArgs) -> RequestVoteReply {
        self.check_term(args.term);
        let mut raft = self.raft.lock().unwrap();
        debug!("request_vote({:?})", args);
        let granted = raft.check_grant(&args);
        info!(
            "{} grant to RV({:?})? = {}",
            raft.self_info(),
            args,
            granted
        );
        if granted {
            raft.reset_election_timer();
            // NOTE：即便没有设置投票者……（就是说，一个节点可以在一个 term 中投多个票）
            // 我们仍旧可以几乎所有情况下通过 2A 和 2B 的所有测试……
            // 为什么没有发生脑裂呢……？
            raft.vote_for(args.candidate_id as usize)
        }
        RequestVoteReply {
            term: raft.term,
            vote_granted: granted,
        }
    }

    /// follower handler for `InstallSnapshot`
    fn do_install_snapshot(&self, args: InstallSnapshotArgs) -> InstallSnapshotReply {
        self.check_term(args.term);
        let term = self.term();
        if args.term < term {
            return InstallSnapshotReply { term };
        }

        let mut raft = self.raft.lock().unwrap();
        // this is from a valid leader, reset election timer.
        raft.reset_election_timer();

        // 防止返回乱序……
        if raft.log.len() > args.last_included_index as usize {
            return InstallSnapshotReply { term };
        }
        raft.log = args.into();

        let last_included_index = raft.log.last_included_index;
        raft.commit_index = last_included_index;
        raft.persist();
        raft.apply_snapshot();
        info!(
            "{} Installed snapshot to index {}.",
            raft.self_info(),
            last_included_index,
        );

        InstallSnapshotReply { term: raft.term }
    }
}

impl RaftService for Node {
    // example RequestVote RPC handler.
    //
    // CAVEATS: Please avoid locking or sleeping here, it may jam the network.
    async_rpc! { request_vote(RequestVoteArgs) -> RequestVoteReply where uses Self::do_request_vote }
    async_rpc! { append_entries(AppendEntriesArgs) -> AppendEntriesReply where uses Self::do_append_entries }
    async_rpc! { install_snapshot(InstallSnapshotArgs) -> InstallSnapshotReply where uses Self::do_install_snapshot }
}
