use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, sync_channel, SyncSender};
use std::thread::sleep;
use std::time::Duration;

use futures::Future;
use futures::sync::mpsc::UnboundedSender;
use rand::Rng;

use labrpc::RpcFuture;

use crate::proto::raftpb::*;
use crate::raft::RaftRole::{CANDIDATE, FOLLOWER, LEADER};

use self::errors::*;
use self::persister::*;

#[cfg(test)]
pub mod config;
pub mod errors;
pub mod persister;
#[cfg(test)]
mod tests;

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

#[derive(Clone, Eq, PartialEq, Copy)]
enum RaftRole {
    LEADER = 0,
    CANDIDATE = 1,
    FOLLOWER = 2,
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
    term: u64,
    voted_for: Option<usize>,
    election_timer: Option<SyncSender<TimerMsg>>,
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
        let mut rf = Raft {
            peers,
            persister,
            me,
            state: Arc::default(),
            apply_ch,
            current_role: FOLLOWER,
            term: 0,
            voted_for: None,
            election_timer: None,
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
    }

    /// restore previously persisted state.
    fn restore(&mut self, data: &[u8]) {
        if data.is_empty() {
            // bootstrap without any state?
            return;
        }
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
                .map_err(|e| {
                    error!("send_request_vote: rpc error {:?} meet.", e);
                    Error::Rpc(e)
                })
                .then(move |res| {
                    info!("send_request_vote: result <<{:?}>> get.", &res);
                    let result = tx.send(res);
                    if let Err(e) = result {
                        warn!("send_request_vote: result of RPC is unused. since: {:?}", e);
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
                        warn!(
                            "send_append_entries: result of RPC is unused. since: {:?}",
                            e
                        );
                    }
                    Ok(())
                }),
        );
        rx
    }

    fn start<M>(&self, command: &M) -> Result<(u64, u64)>
    where
        M: labcodec::Message,
    {
        let index = 0;
        let term = 0;
        let is_leader = true;
        let mut buf = vec![];
        labcodec::encode(command, &mut buf).map_err(Error::Encode)?;
        // Your code here (2B).

        if is_leader {
            Ok((index, term))
        } else {
            Err(Error::NotLeader)
        }
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
        let _ = &self.apply_ch;
        let _ = TimerMsg::Reset;
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

// TODO: 解决这里的线程泄漏问题。（当不再有新消息发送的时候，轮询仍旧会继续。）
fn select<T: Send + 'static>(channels: impl Iterator<Item=Receiver<T>>) -> Receiver<T> {
    let (sx, rx) = channel();
    let channels: Vec<Receiver<T>> = channels.collect();
    std::thread::spawn(move || loop {
        for ch in channels.iter() {
            if let Ok(data) = ch.try_recv() {
                if let Err(_e) = sx.send(data) {
                    debug!("select: select receiver is closed.");
                    return;
                }
            }
            sleep(Duration::from_millis(8))
        }
    });
    rx
}

impl Raft {
    fn make_request_vote_args(&self) -> RequestVoteArgs {
        RequestVoteArgs {
            term: self.term,
            candidate_id: self.me as u64,
            last_log_index: 0,
            last_log_term: 0,
        }
    }

    fn transform_to_candidate(raft: Arc<Mutex<Self>>) {
        std::thread::spawn(move || {
            let mut guard = raft.lock().unwrap();
            guard.current_role = CANDIDATE;
            guard.term += 1;
            let me = guard.me;
            info!("NO{} started a new election of term {}.", me, guard.term);
            guard.voted_for = Some(me);
            let peer_count = guard.peers.len();
            let send_result = (0..peer_count)
                .filter(|i| *i != me)
                .map(|i| guard.send_request_vote(i, &guard.make_request_vote_args()))
                .collect::<Vec<Receiver<_>>>();
            drop(guard);

            let mut vote_count = 1;
            let data_channel = select(send_result.into_iter());
            while let Ok(result) = data_channel.recv_timeout(Duration::from_millis(300)) {
                vote_count += result
                    .map(|reply| if reply.vote_granted { 1 } else { 0 })
                    .unwrap_or(0);
                info!("NO{}: vote_count = {}", me, vote_count);
                if vote_count > peer_count / 2 {
                    info!("NO{} has enough votes!", me);
                    break;
                }
            }

            let mut guard = raft.lock().unwrap();
            if vote_count > peer_count / 2 && guard.current_role == CANDIDATE {
                guard.current_role = LEADER;
                guard.stop_election_timer();
                info!("NO{} is now the leader!", me);
                drop(guard);
                Raft::transform_to_leader(raft);
            }
        });
    }

    fn make_empty_append_entries_for(&self, _server: usize) -> AppendEntriesArgs {
        AppendEntriesArgs { term: self.term }
    }

    fn transform_to_leader(raft: Arc<Mutex<Raft>>) {
        loop {
            let guard = raft.lock().unwrap();
            if guard.current_role != LEADER {
                return;
            }
            let peer_len = guard.peers.len();
            let me = guard.me;
            drop(guard);
            (0..peer_len).filter(|i| *i != me).for_each(|i| {
                let raft = raft.clone();
                std::thread::spawn(move || {
                    let guard = raft.lock().unwrap();
                    let result =
                        guard.send_append_entries(i, &guard.make_empty_append_entries_for(i));
                    match result.recv_timeout(Duration::from_millis(100)) {
                        Err(e) => warn!(
                            "NO{} failed to receive append_entries result to NO{} because: {}",
                            me, i, e
                        ),
                        Ok(info) => debug!("append_entries success! get return value: {:?}", info),
                    };
                });
            });
            sleep(Duration::from_millis(100));
        }
    }

    fn send_to_election_timer(&self, message: TimerMsg) {
        let reset_result = self
            .election_timer
            .as_ref()
            .unwrap_or_else(|| {
                panic!(
                    "NO{} Trying to operate election timer on a raft peer that isn't bind to a node.",
                    self.me
                )
            })
            .send(message);
        if reset_result.is_err() {
            warn!(
                "NO{} Trying to operate election timer on a raft peer with closing timer.",
                self.me
            );
        }
    }

    fn reset_election_timer(&self) {
        if self.current_role == LEADER {
            panic!(
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
        if new_term < self.term {
            panic!(
                "NO{} is set to a lower term, which probably is an error.",
                self.me
            );
        }
        if new_term != self.term {
            info!("NO{} is now set to term {}", self.me, new_term);
            self.voted_for = None;
        }
        self.term = new_term;
    }

    fn transform_to_follower(raft: Arc<Mutex<Raft>>) {
        let (sx, rx) = sync_channel::<TimerMsg>(1);
        let mut guard = raft.lock().unwrap();
        if let Some(sx) = guard.election_timer.take() {
            if sx.send(TimerMsg::Stop).is_ok() {
                warn!("Some path to follower didn't stop the last term's election timer.");
            }
        }
        guard.election_timer = Some(sx.clone());
        guard.current_role = FOLLOWER;
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
                        info!("stop signal received, the timer will stop.");
                        return;
                    }
                }
            }
        });
    }

    fn update_and_check_grant(&mut self, term: u64, candidate_id: u64) -> bool {
        if self.term < term {
            self.update_term(term);
            if self.voted_for.is_none() || self.voted_for == Some(candidate_id as usize) {
                return true;
            }
        }
        false
    }
}

enum TimerMsg {
    Reset,
    Stop,
}

impl Node {
    /// Create a new raft service.
    pub fn new(raft: Raft) -> Node {
        // Your code here.
        // TODO: 为恢复到不同状态的节点配置不同的初始化函数。
        info!("new node NO「{}」started.", raft.me);
        let raft = Arc::new(Mutex::new(raft));
        Raft::transform_to_follower(raft.clone());
        Node { raft: raft.clone() }
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
        let guard = self.raft.lock().unwrap();
        guard.start(command)
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
        guard.current_role == LEADER
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
        let guard = self.raft.lock().unwrap();
        info!("NO{} is dead.", guard.me);
        // Your code here, if desired.
    }
}

impl RaftService for Node {
    // example RequestVote RPC handler.
    //
    // CAVEATS: Please avoid locking or sleeping here, it may jam the network.
    fn request_vote(&self, args: RequestVoteArgs) -> RpcFuture<RequestVoteReply> {
        // TODO: 给他换上一个线程池。
        let (sx, rx) = futures::sync::oneshot::channel::<RequestVoteReply>();
        let raft = self.raft.clone();
        std::thread::spawn(move || {
            let mut guard = raft.lock().unwrap();
            info!("request_vote({:?})", args);

            // TODO: 投票规则到底是什么？！
            let granted = guard.update_and_check_grant(args.term, args.candidate_id);

            info!(
                "NO{} grant to NO{}? = {:?}",
                guard.me, args.candidate_id, granted
            );
            sx.send(RequestVoteReply {
                term: guard.term,
                vote_granted: granted,
            })
        });
        box rx.map_err(|e| panic!("request vote: failed to execute: {}", e))
    }

    fn append_entries(&self, req: AppendEntriesArgs) -> RpcFuture<AppendEntriesReply> {
        let (sx, rx) = futures::sync::oneshot::channel::<AppendEntriesReply>();
        let myself = self.clone();
        std::thread::spawn(move || {
            if req.term >= myself.term() {
                let mut guard = myself.raft.lock().unwrap();
                guard.update_term(req.term);
                if guard.current_role != FOLLOWER {
                    info!("NO{} got append_entries from other with at-least-equals-to term, he eventually has known, he is a follower now.", guard.me);
                    guard.stop_election_timer();
                    drop(guard);
                    Raft::transform_to_follower(myself.raft.clone());
                }
            }
            myself.reset_timer();
            sx.send(AppendEntriesReply {
                term: myself.term(),
                success: true,
            })
        });
        box rx.map_err(|e| panic!("request vote: failed to execute: {}", e))
    }
}
