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
use crate::raft::TimerMsg::Stop;

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

#[derive(Clone, Eq, PartialEq, Copy, Debug)]
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
                    warn!("send_request_vote: network(rpc) error {:?}.", e);
                    Error::Rpc(e)
                })
                .then(move |res| {
                    let result_ok = res.is_ok();
                    let result = tx.send(res);
                    if let Err(e) = result {
                        if result_ok {
                            warn!("send_request_vote: result of RPC is unused. since: {:?}", e);
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

// 有没有比轮询更加好的办法呢？（似乎 Go 语言的 Select 在规模变大之后使用的也是轮询）
fn select<T: Send + 'static>(channels: impl Iterator<Item=Receiver<T>>) -> Receiver<T> {
    use std::sync::mpsc::TryRecvError::*;
    let (sx, rx) = channel();
    let channels: Vec<Receiver<T>> = channels.collect();
    let mut is_available = vec![true; channels.len()];
    let mut available_count = channels.len();
    std::thread::spawn(move || loop {
        if available_count == 0 {
            return;
        }
        for (i, ch) in channels.iter().enumerate() {
            if is_available[i] {
                match ch.try_recv() {
                    Ok(data) => {
                        if let Err(_e) = sx.send(data) {
                            debug!("select: select receiver is closed.");
                            return;
                        }
                    }
                    Err(Disconnected) => {
                        is_available[i] = false;
                        available_count -= 1;
                    }
                    Err(Empty) => {}
                }
            }
        }
        sleep(Duration::from_millis(8))
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
            let term_at_start = guard.term;
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
                && guard.current_role == CANDIDATE
            {
                guard.current_role = LEADER;
                guard.stop_election_timer();
                info!("NO{} is now the leader of term {}.", me, term_at_start);
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
        if self.current_role == LEADER {
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
        }
        self.term = new_term;
    }

    fn transform_to_follower(raft: Arc<Mutex<Raft>>) {
        let (sx, rx) = sync_channel::<TimerMsg>(1);
        let mut guard = raft.lock().unwrap();
        guard.try_send_to_election_timer(Stop);
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

    fn check_grant(&mut self, candidate_id: u64) -> bool {
        self.voted_for.is_none() || self.voted_for == Some(candidate_id as usize)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
enum TimerMsg {
    Reset,
    Stop,
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
        let mut guard = self.raft.lock().unwrap();
        // stop leader timer.
        guard.current_role = FOLLOWER;
        // stop follower timer.
        guard.try_send_to_election_timer(Stop);
        info!("NO{} is dead.", guard.me);
    }

    fn check_term(&self, new_term: u64) {
        if new_term >= self.term() {
            let mut guard = self.raft.lock().unwrap();
            let leader_to_follower = guard.current_role == LEADER && new_term > guard.term;
            let candidate_to_follower = guard.current_role == CANDIDATE;
            guard.update_term(new_term);
            if leader_to_follower || candidate_to_follower {
                info!("NO{}(currentTerm = {}, state = {:?}), get RPC(term = {}), he eventually has known, he is a follower now.",
                      guard.me,
                      guard.term,
                      guard.current_role,
                      new_term);
                drop(guard);
                Raft::transform_to_follower(self.raft.clone());
            }
        }
    }
}

impl RaftService for Node {
    // example RequestVote RPC handler.
    //
    // CAVEATS: Please avoid locking or sleeping here, it may jam the network.
    fn request_vote(&self, args: RequestVoteArgs) -> RpcFuture<RequestVoteReply> {
        let (sx, rx) = futures::sync::oneshot::channel::<RequestVoteReply>();
        let raft = self.raft.clone();
        let myself = self.clone();
        self.rpc_execution_pool.spawn(move || {
            myself.check_term(args.term);
            let mut guard = raft.lock().unwrap();
            debug!("request_vote({:?})", args);
            // TODO: 修改投票规则，使用最后的命令而非简单地比较 term（2B）。
            // TODO：This is buggy. fix it.
            let granted = guard.check_grant(args.candidate_id);
            info!(
                "NO{} grant to NO{}? = {:?}",
                guard.me, args.candidate_id, granted
            );
            if granted {
                guard.reset_election_timer()
            }
            sx.send(RequestVoteReply {
                term: guard.term,
                vote_granted: granted,
            })
                .unwrap_or_else(|args| {
                    warn!(
                        "RPC channel exception, RPC request_vote({:?}) won't be replied.",
                        args
                    )
                });
        });
        box rx.map_err(|e| panic!("request vote: failed to execute: {}", e))
    }

    fn append_entries(&self, req: AppendEntriesArgs) -> RpcFuture<AppendEntriesReply> {
        let (sx, rx) = futures::sync::oneshot::channel::<AppendEntriesReply>();
        let myself = self.clone();
        self.rpc_execution_pool.spawn(move || {
            myself.check_term(req.term);
            if req.term >= myself.term() {
                myself.reset_timer();
            }
            sx.send(AppendEntriesReply {
                term: myself.term(),
                success: true,
            })
                .unwrap_or_else(|req| {
                    warn!(
                        "RPC channel exception, RPC append_entries({:?}) won't be replied.",
                        req
                    )
                });
        });
        box rx.map_err(|e| panic!("request vote: failed to execute: {}", e))
    }
}
