use std::cmp::Ordering;
use std::ops::Deref;
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use futures::sync::mpsc::UnboundedSender;
use futures::Future;
use rand::Rng;
use rayon::{ThreadPool, ThreadPoolBuilder};

use labcodec::{decode, encode};
use labrpc::RpcFuture;

use crate::proto::raftpb::*;
use crate::raft::RaftRole::{Candidate, Follower, Leader};
use crate::raft::TimerMsg::Stop;
use crate::select;

use self::errors::*;
use self::persister::*;

#[cfg(test)]
pub mod config;
pub mod errors;
pub mod persister;
#[cfg(test)]
mod tests;

static PLACE_HOLDER: [u8; 4] = [0xca, 0xfe, 0xba, 0xbe];

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
            leader_append_entries_delay: Duration::from_millis(50),
            latency_tolerance_factor: 2.0,
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

struct SentAppendEntriesRequest {
    follower: usize,
    request: AppendEntriesArgs,
    response: Receiver<Result<AppendEntriesReply>>,
}

impl
    From<(
        usize,
        AppendEntriesArgs,
        Receiver<Result<AppendEntriesReply>>,
    )> for SentAppendEntriesRequest
{
    fn from(
        origin: (
            usize,
            AppendEntriesArgs,
            Receiver<Result<AppendEntriesReply>>,
        ),
    ) -> Self {
        SentAppendEntriesRequest {
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
    // Your data here (2A, 2B, 2C).
    // Look at the paper's Figure 2 for a description of what
    // state a Raft server must maintain.
    apply_ch: UnboundedSender<ApplyMsg>,
    current_role: RaftRole,
    election_timer: Option<SyncSender<TimerMsg>>,

    // stored state.
    term: u64,
    voted_for: Option<usize>,
    log: Vec<LogEntry>,

    // in-memory state
    commit_index: u64,
    last_applied: u64,

    // leader state
    leader_state: Option<LeaderState>,
    leader_execution_pool: ThreadPool,

    // misc
    extra: RaftConfig,
}

#[derive(Clone, Debug)]
struct LogEntry {
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
}

impl LeaderState {
    fn by_raft(raft: &Raft) -> Self {
        LeaderState {
            next_index: vec![raft.last_log_index() + 1; raft.peers.len()],
            match_index: vec![0; raft.peers.len()],
        }
    }
}

impl PersistedStatus {
    fn by_raft(raft: &Raft) -> Self {
        PersistedStatus {
            current_term: raft.term,
            voted_for: raft.voted_for.iter().map(|x| *x as u64).collect(),
            logs: raft.log.iter().cloned().map(Into::into).collect(),
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
            log: vec![LogEntry::new(Vec::from(&PLACE_HOLDER[..]), 0)],
            commit_index: 0,
            last_applied: 0,
            leader_state: None,
            leader_execution_pool: ThreadPoolBuilder::new()
                .thread_name(move |n| format!("[`{}`] leader execution worker ({})", me, n))
                .num_threads(((peer_count as f64) * config.latency_tolerance_factor) as usize)
                .build()
                .unwrap(),
            extra: config,
        };

        // initialize from state persisted before a crash
        rf.restore(&raft_state);

        rf
    }

    /// save Raft's persistent state to stable storage,
    /// where it can later be retrieved after a crash and restart.
    /// see paper's Figure 2 for a description of what should be persistent.
    fn persist(&mut self) {
        // Your code here (2C).
        // Example:
        // labcodec::encode(&self.xxx, &mut data).unwrap();
        // labcodec::encode(&self.yyy, &mut data).unwrap();
        // self.persister.save_raft_state(data);
        let persisted = PersistedStatus::by_raft(self);
        let mut buf = vec![];
        encode(&persisted, &mut buf).unwrap();
        self.persister.save_raft_state(buf);
    }

    /// restore previously persisted state.
    fn restore(&mut self, data: &[u8]) {
        if data.is_empty() {
            info!("{} bootstrap without any state!", self.self_info());
            return;
        }
        match decode::<PersistedStatus>(data) {
            Ok(state) => {
                info!("{} restored to: {:?}", self.self_info(), state);
                self.term = state.current_term;
                self.log = state.logs.into_iter().map(Into::into).collect();
                self.voted_for = state.voted_for.first().map(|x| *x as usize);
            }
            Err(e) => {
                panic!("Failed to decode: {:?}", e);
            }
        };
        // Your code here (2C).
        // Example:
        // match labcodec::decode(data) {
        //     Ok(o) => {
        //         self.xxx = o.xxx;
        //         self.yyy = o.yyy;
        //     }
        //     Err(e) => {
        //         panic!("{:?}", e);
        //     }
        // }
    }

    /// example code to send a RequestVote RPC to a server.
    /// server is the index of the target server in peers.
    /// expects RPC arguments in args.
    ///
    /// The labrpc package simulates a lossy network, in which servers
    /// may be unreachable, and in which requests and replies may be lost.
    /// This method sends a request and waits for a reply. If a reply arrives
    /// within a timeout interval, This method returns Ok(_); otherwise
    /// this method returns Err(_). Thus this method may not return for a while.
    /// An Err(_) return can be caused by a dead server, a live server that
    /// can't be reached, a lost request, or a lost reply.
    ///
    /// This method is guaranteed to return (perhaps after a delay) *except* if
    /// the handler function on the server side does not return.  Thus there
    /// is no need to implement your own timeouts around this method.
    ///
    /// look at the comments in ../labrpc/src/lib.rs for more details.
    fn send_request_vote(
        &self,
        server: usize,
        args: &RequestVoteArgs,
    ) -> Receiver<Result<RequestVoteReply>> {
        let peer = &self.peers[server];
        let (tx, rx) = channel::<Result<RequestVoteReply>>();
        peer.spawn(
            peer.request_vote(&args)
                .map_err(Error::Rpc)
                .then(move |res| {
                    let result_ok = res.is_ok();
                    let result = tx.send(res);
                    if let Err(e) = result {
                        if result_ok {
                            debug!(
                                "send_request_vote: result of RPC({:?}) is unused. since: {:?}",
                                e.0, e
                            );
                        }
                    }
                    Ok(())
                }),
        );
        rx
    }

    fn send_append_entries(
        &self,
        server: usize,
        args: &AppendEntriesArgs,
    ) -> Receiver<Result<AppendEntriesReply>> {
        let peer = &self.peers[server];
        let (tx, rx) = channel::<Result<AppendEntriesReply>>();
        peer.spawn(
            peer.append_entries(&args)
                .map_err(Error::Rpc)
                .then(move |res| {
                    let result = tx.send(res);
                    if let Err(e) = result {
                        debug!(
                            "send_append_entries: result of RPC({:?}) is unused. since: {:?}",
                            e.0, e
                        );
                    }
                    Ok(())
                }),
        );
        rx
    }

    fn self_info(&self) -> String {
        format!(
            "[(`{}`@term{}), {:?}]",
            self.me, self.term, self.current_role
        )
    }

    fn make_log(&self, data: Vec<u8>) -> LogEntry {
        LogEntry::new(data, self.term)
    }

    fn log_info(&self) -> String {
        format!("{:?}", self.log.iter().map(|e| e.term).collect::<Vec<_>>())
    }

    fn start<M>(&mut self, command: &M) -> Result<(u64, u64)>
    where
        M: labcodec::Message,
    {
        let is_leader = self.current_role == Leader;
        if !is_leader {
            return Err(Error::NotLeader);
        }

        let mut buf = vec![];
        info!(
            "{} get command: {:?}(logs = {:?})",
            self.self_info(),
            command,
            self.log_info()
        );
        labcodec::encode(command, &mut buf).map_err(Error::Encode)?;
        let entry = self.make_log(buf);
        self.log.push(entry);
        self.persist();

        let index = self.last_log_index();
        let term = self.term;
        Ok((index, term))
    }
}

impl Raft {
    /// Only for suppressing deadcode warnings.
    #[doc(hidden)]
    pub fn __suppress_deadcode(&mut self) {
        let _ = self.start(&0);
        let _ = self.send_request_vote(0, &Default::default());
        self.persist();
        let _ = &self.state;
        let _ = &self.me;
        let _ = &self.persister;
        let _ = &self.peers;

        // user added.
        let _ = &self.make_empty_append_entries();
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
    rpc_execution_pool: Arc<rayon::ThreadPool>,
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

fn mid<T: Ord>(items: &mut [T]) -> &T {
    items.sort_by(reverse_order);
    let mid = (items.len() - 1) / 2;
    &items[mid]
}

impl Raft {
    fn is_leader(&self) -> bool {
        self.current_role == Leader
    }

    fn get_term_starts_at(&self, term: u64, from: usize) -> usize {
        let mut n = from;
        while self.log[n].term == term {
            n -= 1;
        }
        n + 1
    }

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

    fn make_apply_message(&self, index: u64) -> ApplyMsg {
        let log = &self.log[index as usize];
        ApplyMsg {
            command_valid: true,
            command: log.data.clone(),
            command_index: index,
        }
    }

    fn apply_logs(&mut self) {
        for i in (self.last_applied + 1)..=(self.commit_index) {
            self.apply_ch
                .unbounded_send(self.make_apply_message(i))
                .expect("fetal: failed to send to apply ch.");
        }
        self.last_applied = self.commit_index;
    }

    fn leader_commit_logs(&mut self) {
        let next = self.next_commit_index();
        // 5.4.2: NEVER commit log entries from previous terms by counting replicas.
        if next > self.commit_index && self.log[next as usize].term == self.term {
            self.commit_index = next;
            self.apply_logs();
        }
    }

    fn last_log_index(&self) -> u64 {
        self.log.len() as u64 - 1
    }

    fn last_log_term(&self) -> u64 {
        self.log.last().map(|e| e.term).unwrap_or(0)
    }

    fn make_request_vote_args(&self) -> RequestVoteArgs {
        RequestVoteArgs {
            term: self.term,
            candidate_id: self.me as u64,
            last_log_index: self.last_log_index(),
            last_log_term: self.last_log_term(),
        }
    }

    // TODO: 错综复杂的 Mutex 获取、释放关系……贸然重构或许会有些危险……
    // TODO：这种方式一开始就是 error-prone 的吗？
    fn transform_to_candidate(raft: Arc<Mutex<Self>>) {
        std::thread::spawn(move || {
            let mut guard = raft.lock().unwrap();
            guard.current_role = Candidate;
            // make the borrow checker happy.
            let old_term = guard.term;
            guard.update_term(old_term + 1);
            let term_at_start = guard.term;
            let me = guard.me;
            info!("NO{} started a new election of term {}.", me, guard.term);
            guard.vote_for(me);
            let peer_count = guard.peers.len();
            let send_result = (0..peer_count)
                .filter(|i| *i != me)
                .map(|i| guard.send_request_vote(i, &guard.make_request_vote_args()))
                .collect::<Vec<Receiver<_>>>();
            drop(guard);
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
                if vote_count > peer_count / 2 {
                    info!("NO{} has enough votes at term {}!", me, term_at_start);
                    break;
                }
            }

            let mut guard = raft.lock().unwrap();
            if vote_count > peer_count / 2
                // ensure that we didn't start another term of election...
                && guard.term == term_at_start
                // ensure that there isn't a leader...
                && guard.current_role == Candidate
            {
                guard.current_role = Leader;
                guard.stop_election_timer();
                info!("NO{} is now the leader of term {}.", me, term_at_start);
                drop(guard);
                Raft::transform_to_leader(raft);
            }
        });
    }

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
        } else {
            let next_index = response.conflicted_term_starts_at as usize;
            if next_index == 0xcafe_babe {
                panic!("A debug magic number appears, which might means InvalidLeader message has handled by incorrect way.\n\
                Debug info: ({:?}) => {:?} self = {}", request, response, self_info);
            }
            let can_match = self
                .log
                .get(next_index)
                .map(|idx| idx.term == response.conflicted_term)
                .unwrap_or(false);
            let real_next = if can_match {
                next_index
            } else {
                next_index - 1
            } as u64;
            // don't send placeholder!
            leader_state.next_index[follower] = Ord::max(1, real_next);
        }
    }

    fn handle_append_entries(
        raft_lock: Arc<Mutex<Self>>,
        request: &AppendEntriesArgs,
        response: &AppendEntriesReply,
        follower: usize,
    ) {
        let mut raft = raft_lock.lock().unwrap();
        let raft_info = raft.self_info();
        info!(
            "[{}] append_entries({:?}) => {:?}",
            raft_info, request, response
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

    fn make_empty_append_entries(&self) -> AppendEntriesArgs {
        AppendEntriesArgs {
            term: self.term,
            leader_id: self.me as u64,
            prev_log_index: 0,
            prev_log_term: 0,
            entries: vec![],
            leader_commit: self.commit_index,
        }
    }

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
            prev_log_term: self.log[next_index - 1].term,
            entries: self.log[next_index..]
                .iter()
                .cloned()
                .map(Into::into)
                .collect(),
            leader_commit: self.commit_index,
        }
    }

    fn send_append_entries_requests<F>(&self, mut factory: F) -> Vec<SentAppendEntriesRequest>
    where
        F: FnMut(&Raft, usize) -> AppendEntriesArgs,
    {
        let peer_len = self.peers.len();
        (0..peer_len)
            .filter(|i| *i != self.me)
            .map(|i| {
                let request = factory(self, i);
                let response = self.send_append_entries(i, &request);
                SentAppendEntriesRequest::from((i, request, response))
            })
            .collect()
    }

    // ……我们用了与 paper 中不同的办法来发送 AppendEntries.
    // 不是在收到客户端请求后"立即"发起 AppendEntries，
    // 而是在发送心跳包的同时带上这些信息。
    // 这样做能够在符合 5.3 的约定（只是……延迟会变高）的同时让实现更加简单一些。
    // **似乎**可视化的 Raft 也是这样做的。
    // 那些约定摘录如下：
    // - Leader 需要在收到请求时将请求录入 log，并且并行地发送 AppendEntries 请求。
    // - 如果 Follower 不可用，那么需要无限地重试。
    // 唯一的问题是我们违反了 5.2 中关于心跳包的问题，但是其原文是，
    // 在空闲期间发送空心跳包。
    // 另：此处的风格更加接近一种 "micro-batch"，即统计一段时间内所有的 entries 然后一次发送。
    // 这样的做法会增加客户端的延迟（客户平均至少需要等待半个 `AppendEntries` 的周期才能获得确定提交的相应。）
    fn transform_to_leader(raft_lock: Arc<Mutex<Raft>>) {
        let mut guard = raft_lock.lock().unwrap();
        guard.try_send_to_election_timer(Stop);
        guard.leader_state = Some(LeaderState::by_raft(guard.deref()));
        drop(guard);
        loop {
            let mut raft = raft_lock.lock().unwrap();
            if !raft.is_leader() {
                break;
            }
            raft.leader_commit_logs();
            let raft_info = raft.self_info();

            info!("{}: leader_state = {:?}", raft_info, raft.leader_state);
            let requests =
                raft.send_append_entries_requests(|raft, i| raft.make_append_entries_for(i));
            drop(raft);
            Raft::spawn_append_entries_handler(raft_lock.clone(), requests.into_iter());
            sleep(Duration::from_millis(100));
        }
        let mut guard = raft_lock.lock().unwrap();
        guard.leader_state = None;
    }

    fn spawn_append_entries_handler(
        raft_lock: Arc<Mutex<Self>>,
        requests: impl Iterator<Item = SentAppendEntriesRequest>,
    ) {
        let raft = raft_lock.lock().unwrap();
        requests.for_each(|req| {
            let raft_info = raft.self_info();
            let raft_lock = raft_lock.clone();
            let delay = raft.extra.get_timeout();
            raft.leader_execution_pool.spawn({
                let raft_lock = raft_lock.clone();
                move || {
                    match req.response.recv_timeout(delay) {
                        Err(_e) => debug!(
                            "{} failed to receive append_entries result to NO{} because timeout.",
                            raft_info, req.follower
                        ),
                        Ok(Err(e)) => {
                            info!(
                                "{} Failed to get result of {:?}, because: {}",
                                raft_info, req.request, e
                            );
                        }
                        Ok(Ok(info)) => {
                            Raft::handle_append_entries(
                                raft_lock.clone(),
                                &req.request,
                                &info,
                                req.follower,
                            );
                        }
                    };
                }
            });
        });
    }

    fn try_send_to_election_timer(&self, message: TimerMsg) -> bool {
        self.election_timer
            .as_ref()
            .map(|sx| sx.send(message))
            .map(|result| result.map(|_| true).unwrap_or(false))
            .unwrap_or(false)
    }

    fn send_to_election_timer(&self, message: TimerMsg) {
        if !self.try_send_to_election_timer(message) {
            info!(
                "NO{} send_to_election_timer({:?}): failed try.",
                self.me, message
            );
        }
    }

    fn reset_election_timer(&self) {
        if self.is_leader() {
            warn!(
                "NO{} Trying to reset election timer on a leader node.",
                self.me
            );
        }

        self.send_to_election_timer(TimerMsg::Reset);
    }

    fn stop_election_timer(&self) {
        self.send_to_election_timer(TimerMsg::Stop);
    }

    fn generate_election_timeout() -> Duration {
        let range = rand::thread_rng().gen_range(150, 300);
        Duration::from_millis(range)
    }

    fn update_term(&mut self, new_term: u64) {
        assert!(
            self.term <= new_term,
            "NO{}(currentTerm = {}) is set to a lower term({}), which is probably an error.",
            self.me,
            self.term,
            new_term
        );
        if new_term != self.term {
            info!("NO{} is now set to term {}", self.me, new_term);
            self.voted_for = None;
            self.persist();
        }
        self.term = new_term;
    }

    fn transform_to_follower(raft: Arc<Mutex<Raft>>) {
        let (sx, rx) = sync_channel::<TimerMsg>(1);
        let mut guard = raft.lock().unwrap();
        guard.try_send_to_election_timer(Stop);
        guard.election_timer = Some(sx.clone());
        guard.current_role = Follower;
        std::thread::spawn({
            let raft = raft.clone();
            move || loop {
                let message = rx.recv_timeout(Raft::generate_election_timeout());
                match message {
                    Err(_) => {
                        Raft::transform_to_candidate(raft.clone());
                    }
                    Ok(TimerMsg::Reset) => debug!("Reset election timeout!"),
                    Ok(TimerMsg::Stop) => {
                        debug!("stop signal received, the timer will stop.");
                        return;
                    }
                }
            }
        });
    }

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
    /// returns `true` if the raft peer become follower since this function.
    fn check_term(raft: Arc<Mutex<Raft>>, new_term: u64) -> bool {
        let mut guard = raft.lock().unwrap();
        if new_term >= guard.term {
            let old_term = guard.term;
            let leader_to_follower = guard.is_leader() && new_term > old_term;
            let candidate_to_follower = guard.current_role == Candidate;
            guard.update_term(new_term);
            if leader_to_follower || candidate_to_follower {
                info!("NO{}(currentTerm = {}, state = {:?}), get RPC(term = {}), he eventually has known, he is a follower now.",
                      guard.me,
                      old_term,
                      guard.current_role,
                      new_term);
                drop(guard);
                Raft::transform_to_follower(raft.clone());
                return true;
            }
        }
        false
    }

    fn check_and_trunc_log(&mut self, base: usize, entries: &[LogEntry]) -> usize {
        for (offset, remote) in entries.iter().enumerate() {
            let local = self.log.get(base + offset);
            if local.is_none() || local.unwrap().term != remote.term {
                self.log.truncate(base + offset);
                return offset;
            }
        }
        entries.len()
    }

    fn vote_for(&mut self, candidate: usize) {
        let voted = self.voted_for.map(|x| candidate != x).unwrap_or(false);
        if voted {
            warn!(
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

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
enum TimerMsg {
    Reset,
    Stop,
}

enum FailedAppendEntries {
    ConflictedEntry {
        conflicted_term: u64,
        conflicted_term_starts_at: u64,
    },
    InvalidLeader,
}

impl Node {
    /// Create a new raft service.
    pub fn new(raft: Raft) -> Node {
        // Your code here.
        // TODO: 为恢复到不同状态的节点配置不同的初始化函数。(2C)
        info!("new node NO「{}」started.", raft.me);
        let pool = rayon::ThreadPoolBuilder::new()
            // Assume that every peer sends rpc to me.
            .num_threads(raft.peers.len() - 1)
            .thread_name(|i| format!("rpc executor({})", i))
            .build()
            .unwrap_or_else(|e| panic!("fetal: failed to build thread pool: {}", e));
        let raft = Arc::new(Mutex::new(raft));
        Raft::transform_to_follower(raft.clone());
        Node {
            raft: raft.clone(),
            rpc_execution_pool: Arc::new(pool),
        }
    }

    /// the service using Raft (e.g. a k/v server) wants to start
    /// agreement on the next command to be appended to Raft's log. if this
    /// server isn't the leader, returns [`Error::NotLeader`]. otherwise start
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

        // 2. Reply false if log doesn't match.
        let mut raft = self.raft.lock().unwrap();
        let prev_log_index = args.prev_log_index as usize;
        let entry = raft.log.get(prev_log_index);
        let term_matches = entry.map(|e| e.term == args.prev_log_term).unwrap_or(false);
        if !term_matches {
            if entry.is_none() {
                return Err(FailedAppendEntries::ConflictedEntry {
                    conflicted_term: 0,
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
        // Anyway, persist it.
        if log_changed {
            raft.persist();
        }

        // 5. Set commit index.
        if args.leader_commit > raft.commit_index {
            let next = Ord::min(args.leader_commit, raft.last_log_index());
            raft.commit_index = next;
            raft.apply_logs();
        }

        Ok(())
    }

    /// This function will be executed in a OS thread, don't worry even it blocks.
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
        // TODO: 这儿似乎有一些风险，如果发生活锁，那么请来看看这里。
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
}

impl RaftService for Node {
    // example RequestVote RPC handler.
    //
    // CAVEATS: Please avoid locking or sleeping here, it may jam the network.
    fn request_vote(&self, args: RequestVoteArgs) -> RpcFuture<RequestVoteReply> {
        let (sx, rx) = futures::sync::oneshot::channel::<RequestVoteReply>();
        let myself = self.clone();
        self.rpc_execution_pool.spawn(move || {
            sx.send(myself.do_request_vote(args))
                .unwrap_or_else(|args| {
                    warn!(
                        "RPC channel exception, RPC request_vote({:?}) won't be replied.",
                        args
                    )
                });
        });
        Box::new(rx.map_err(|e| panic!("request vote: failed to execute: {}", e)))
    }

    fn append_entries(&self, req: AppendEntriesArgs) -> RpcFuture<AppendEntriesReply> {
        let (sx, rx) = futures::sync::oneshot::channel::<AppendEntriesReply>();
        let myself = self.clone();
        self.rpc_execution_pool.spawn(move || {
            sx.send(myself.do_append_entries(req))
                .unwrap_or_else(|req| {
                    warn!(
                        "RPC channel exception, RPC append_entries({:?}) won't be replied.",
                        req
                    )
                });
        });
        Box::new(rx.map_err(|e| panic!("request vote: failed to execute: {}", e)))
    }
}
