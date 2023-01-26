use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use std::u64;

use rand::Rng;

#[cfg(test)]
pub mod config;
pub mod errors;
pub mod persister;

#[cfg(test)]
mod paper_tests;
#[cfg(test)]
mod tests;

mod append_entries;
mod heartbeat;
mod msg;
mod state;
mod vote;

use self::errors::*;
use self::persister::*;

use crate::proto::{raftpb::*, ProtoMessage};

/// As each Raft peer becomes aware that successive log entries are committed,
/// the peer should send an `ApplyMsg` to the service (or tester) on the same
/// server, via the `apply_ch` passed to `Raft::new`.
pub enum ApplyMsg {
    Command {
        data: Vec<u8>,
        index: u64,
    },
    // For 2D:
    Snapshot {
        data: Vec<u8>,
        term: u64,
        index: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateType {
    Follower,
    Candidate,
    Leader,
}

impl Default for StateType {
    fn default() -> StateType {
        StateType::Follower
    }
}

/// State of a raft peer.
#[derive(Default, Clone, Debug)]
pub struct State {
    pub term: u64,
    pub is_leader: bool,
    pub state_type: StateType,
    pub leader_id: Option<u64>,
    pub voted_for: Option<u64>,
    pub votes: HashMap<u64, bool>,
}

impl State {
    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.term
    }
    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        self.state_type == StateType::Leader
    }

    /// The current state type of this peer
    pub fn state_type(&self) -> StateType {
        self.state_type
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub from: Option<u64>,
    pub to: Option<u64>,
    pub message: ProtoMessage,
}

impl From<ProtoMessage> for Message {
    fn from(m: ProtoMessage) -> Message {
        Message {
            from: None,
            to: None,
            message: m,
        }
    }
}

// A single Raft peer.
pub struct Raft {
    // RPC end points of all peers
    peers: Arc<Vec<RaftClient>>,
    // Object to hold this peer's persisted state
    persister: Mutex<Box<dyn Persister>>,
    // this peer's index into peers[]
    me: usize,
    pub state: State,
    // Your data here (2A, 2B, 2C).
    // Look at the paper's Figure 2 for a description of what
    // state a Raft server must maintain.

    // msgs need to send
    msgs: Arc<Mutex<VecDeque<Message>>>,

    // heartbeat interval, should send
    heartbeat_timeout: usize,

    // baseline of election interval
    election_timeout: usize,

    // number of ticks since it reached last heartbeatTimeout.
    // only leader keeps heartbeatElapsed.
    heartbeat_elapsed: usize,

    // Ticks since it reached last electionTimeout when it is leader or candidate.
    // Number of ticks since it reached last electionTimeout or received a
    // valid message from current leader when it is a follower.
    election_elapsed: usize,

    // randomized election timeout
    randomized_election_timeout: usize,

    worker_handler: Option<thread::JoinHandle<()>>,

    pub rx: Option<Receiver<errors::Result<Message>>>,
    tx: Sender<errors::Result<Message>>,
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
        _apply_ch: UnboundedSender<ApplyMsg>,
    ) -> Raft {
        let raft_state = persister.raft_state();

        let election_timeout = 15; // 750ms-1500ms
        let heartbeat_timeout = 5; // 250ms

        let msgs = Mutex::new(VecDeque::<Message>::new());
        let ref_msgs = Arc::new(msgs);
        let ref_peers = Arc::new(peers);
        let (tx, rx) = mpsc::channel::<errors::Result<Message>>();

        // let ref_persister = Arc::new(persister);
        // Your initialization code here (2A, 2B, 2C).
        let mut rf = Raft {
            peers: ref_peers.clone(),
            persister: Mutex::new(persister),
            me,
            state: State::default(),
            election_elapsed: 0,
            heartbeat_elapsed: 0,
            // unit tick
            heartbeat_timeout,
            election_timeout,
            randomized_election_timeout: rand::thread_rng()
                .gen_range(election_timeout, 2 * election_timeout),
            msgs: ref_msgs.clone(),
            worker_handler: None,
            rx: rx.into(),
            tx,
        };

        info!(
            "N{} randomized_election_timeout :{}",
            rf.me, rf.randomized_election_timeout
        );

        rf.run_worker(ref_peers, ref_msgs);
        // initialize from state persisted before a crash
        rf.restore(&raft_state);

        // crate::your_code_here((rf, apply_ch))

        rf
    }

    fn run_worker(&self, ref_peers: Arc<Vec<RaftClient>>, ref_msgs: Arc<Mutex<VecDeque<Message>>>) {
        let tx = self.tx.clone();
        thread::spawn(move || loop {
            while let Some(m) = ref_msgs.lock().unwrap().pop_front() {
                match m.message {
                    ProtoMessage::AppendEntriesArgs(args) => {
                        let target = m.to.unwrap() as usize;
                        let peer = ref_peers.as_ref().get(target).unwrap();
                        let peer_clone = peer.clone();
                        let tx_clone = tx.clone();
                        let to = m.to;
                        let from = m.from;
                        peer.spawn(async move {
                            debug!("N{:?} -> N{:?} sending append_entries{:?}", from, to, args);
                            let reply = peer_clone.append_entries(&args).await.map_err(Error::Rpc);
                            match reply {
                                Ok(r) => {
                                    let _ = tx_clone.send(Ok(Message {
                                        from: to,
                                        to: from,
                                        message: ProtoMessage::AppendEntriesReply(r),
                                    }));
                                }
                                Err(err) => {
                                    let _ = tx_clone.send(Err(err));
                                }
                            }
                        });
                    }
                    ProtoMessage::HeartbeatArgs(args) => {
                        let target = m.to.unwrap() as usize;
                        let peer = ref_peers.as_ref().get(target).unwrap();
                        let peer_clone = peer.clone();
                        let tx_clone = tx.clone();
                        let from = m.from;
                        let to = m.to;

                        peer.spawn(async move {
                            // debug!("N{:?} -> N{:?} sending heartbeat{:?}", from, to, args);
                            let reply = peer_clone.heartbeat(&args).await.map_err(Error::Rpc);
                            match reply {
                                Ok(r) => {
                                    let _ = tx_clone.send(Ok(Message {
                                        from: to,
                                        to: from,
                                        message: ProtoMessage::HeartbeatReply(r),
                                    }));
                                }
                                Err(err) => {
                                    debug!(
                                        "N{:?} -> N{:?} sent heartbeat but received error{:?}",
                                        from, to, err
                                    );
                                    let _ = tx_clone.send(Err(err));
                                }
                            }
                        });
                    }
                    ProtoMessage::RequestVoteArgs(args) => {
                        let target = m.to.unwrap() as usize;
                        let peer = ref_peers.as_ref().get(target).unwrap();
                        let peer_clone = peer.clone();
                        let tx_clone = tx.clone();
                        let from = m.from;
                        let to = m.to;

                        peer.spawn(async move {
                            // debug!("N{:?} -> N{:?} sending request_vote{:?}", from, to, args);
                            let reply = peer_clone.request_vote(&args).await.map_err(Error::Rpc);

                            match reply {
                                Ok(r) => {
                                    let _ = tx_clone.send(Ok(Message {
                                        from: to,
                                        to: from,
                                        message: ProtoMessage::RequestVoteReply(r),
                                    }));
                                }
                                Err(err) => {
                                    let _ = tx_clone.send(Err(err));
                                }
                            }
                        });
                    }
                    _ => {
                        debug!("run_worker: unexpected msg {:?}", m);
                    }
                }
            }

            sleep(Duration::from_millis(5));
        });
    }

    fn update_config(&mut self, election_timeout: Option<usize>, heartbeat_timeout: Option<usize>) {
        if let Some(election_timeout) = election_timeout {
            self.election_elapsed = election_timeout;
        }
        if let Some(heartbeat_timeout) = heartbeat_timeout {
            self.heartbeat_timeout = heartbeat_timeout;
        }
    }

    /// tick advances the internal logical clock by a single tick.
    ///
    /// safe to modify data due to tick being called by a single thread
    fn tick(&mut self) {
        match self.state.state_type {
            StateType::Leader => {
                self.heartbeat_elapsed += 1;
                if self.heartbeat_elapsed >= self.heartbeat_timeout {
                    self.bcastbeat();
                    self.heartbeat_elapsed = 0;
                }
            }
            StateType::Candidate => {
                self.election_elapsed += 1;
                if self.election_elapsed >= self.randomized_election_timeout {
                    debug!("Timeout: N{} reach timeout, re-elect", self.me);
                    // sends a local message to self
                    self.step(
                        Message {
                            message: ProtoMessage::MsgHup,
                            from: None,
                            to: None,
                        },
                        None,
                    )
                }
            }
            StateType::Follower => {
                self.election_elapsed += 1;
                if self.election_elapsed >= self.randomized_election_timeout {
                    debug!("Timeout: N{} reach timeout, begin election", self.me);
                    // sends a local message to self
                    self.step(
                        Message {
                            message: ProtoMessage::MsgHup,
                            from: None,
                            to: None,
                        },
                        None,
                    )
                }
            }
        }
    }

    fn step(&mut self, msg: Message, sender: Option<Sender<ProtoMessage>>) {
        match self.state.state_type {
            StateType::Follower => match msg.message {
                ProtoMessage::RequestVoteArgs(args) => {
                    if args.term >= self.state.term {
                        // handle
                        let _ = sender
                            .unwrap()
                            .send(self.handle_request_vote(args.candidate_id, args));
                    } else {
                        //rejects the vote request
                        debug!(
                            "handle request_vote: N{:?} -> N{:?} rejects, latest term {} > {}",
                            args.candidate_id, self.me, self.state.term, args.term,
                        );

                        let _ = sender.unwrap().send(ProtoMessage::RequestVoteReply(
                            RequestVoteReply {
                                term: self.state.term,
                                vote_granted: false,
                            },
                        ));
                    }
                }
                ProtoMessage::RequestVoteReply(reply) => {
                    if reply.term > self.state.term {
                        debug!(
                            "N{}(follower) received higher term {} > {} vote response from N{:?}",
                            self.me, reply.term, self.state.term, msg.from,
                        );
                        self.become_follower(reply.term, None)
                    }
                }
                ProtoMessage::MsgHup => {
                    info!(
                        "N{} received msg hup message, term: {}",
                        self.me, self.state.term,
                    );
                    self.become_candidate();
                    self.request_vote();
                }
                ProtoMessage::HeartbeatArgs(args) => {
                    if args.term >= self.state.term {
                        self.update_state_from_heartbeat(args.leader_id, &args);
                    }
                    let _ = sender
                        .unwrap()
                        .send(self.handle_heartbeat(args.leader_id, args));
                }
                _ => {
                    debug!("N {} follower unexpcted msg {:?}", self.me, msg.message)
                }
            },
            StateType::Candidate => match msg.message {
                ProtoMessage::RequestVoteArgs(args) => {
                    if args.term >= self.state.term {
                        // handle
                        let _ = sender
                            .unwrap()
                            .send(self.handle_request_vote(args.candidate_id, args));
                    } else {
                        //rejects the vote request
                        debug!(
                            "handle request_vote: N{:?} -> N{:?} rejects, latest term {} > {}",
                            args.candidate_id, self.me, self.state.term, args.term,
                        );
                        let _ = sender.unwrap().send(ProtoMessage::RequestVoteReply(
                            RequestVoteReply {
                                term: self.state.term,
                                vote_granted: false,
                            },
                        ));
                    }
                }
                ProtoMessage::HeartbeatArgs(args) => {
                    if args.term >= self.state.term {
                        self.become_follower(args.term, args.leader_id.into());
                    }
                    let _ = sender
                        .unwrap()
                        .send(self.handle_heartbeat(args.leader_id, args));
                }
                ProtoMessage::RequestVoteReply(reply) => {
                    if reply.term >= self.state.term {
                        self.handle_request_vote_reply(msg.from.unwrap(), reply);
                    }
                }
                ProtoMessage::MsgHup => {
                    info!(
                        "N{} received msg hup message, term: {}",
                        self.me, self.state.term,
                    );
                    self.become_candidate();
                    self.request_vote();
                }
                ProtoMessage::AppendEntriesArgs(args) => {
                    if args.term >= self.state.term {
                        self.become_follower(args.term, args.leader_id.into());
                        let _ = sender.unwrap().send(self.handle_append_entries(args));
                    } else {
                        let _ = sender.unwrap().send(ProtoMessage::AppendEntriesReply(
                            AppendEntriesReply {
                                term: self.state.term,
                                success: false,
                            },
                        ));
                    }
                }
                _ => {
                    debug!("N{} candidate unexpcted msg {:?}", self.me, msg.message)
                }
            },
            StateType::Leader => match msg.message {
                ProtoMessage::RequestVoteArgs(args) => {
                    if args.term >= self.state.term {
                        // handle
                        let _ = sender
                            .unwrap()
                            .send(self.handle_request_vote(args.candidate_id, args));
                    } else {
                        //rejects the vote request
                        debug!(
                            "handle request_vote: N{:?} -> N{:?} rejects, latest term {} > {}",
                            args.candidate_id, self.me, self.state.term, args.term,
                        );

                        let _ = sender.unwrap().send(ProtoMessage::RequestVoteReply(
                            RequestVoteReply {
                                term: self.state.term,
                                vote_granted: false,
                            },
                        ));
                    }
                }
                ProtoMessage::MsgBeat => self.bcastbeat(),
                ProtoMessage::HeartbeatArgs(args) => {
                    if args.term > self.state.term {
                        // updates term from the heartbeats
                        self.update_state_from_heartbeat(args.leader_id, &args);

                        let _ = sender
                            .unwrap()
                            .send(self.handle_heartbeat(args.leader_id, args));
                    }
                    // ignores
                }
                ProtoMessage::HeartbeatReply(reply) => {
                    if reply.term == self.state.term {
                        self.handle_heartbeat_reply(reply);
                    } else if reply.term > self.state.term {
                        self.become_follower(reply.term, None);
                    }
                }
                ProtoMessage::RequestVoteReply(reply) => {
                    if reply.term > self.state.term {
                        debug!(
                            "N{}(leader) received higher term {} > {} vote response from N{:?}",
                            self.me, reply.term, self.state.term, msg.from
                        );
                        self.become_follower(reply.term, None)
                    }
                }
                _ => {
                    debug!("N{} leader unexpcted msg {:?}", self.me, msg.message)
                }
            },
        }
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

    fn start<M>(&mut self, command: &M) -> Result<(u64, u64)>
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

    fn cond_install_snapshot(
        &mut self,
        last_included_term: u64,
        last_included_index: u64,
        snapshot: &[u8],
    ) -> bool {
        // Your code here (2D).
        crate::your_code_here((last_included_term, last_included_index, snapshot));
    }

    fn snapshot(&mut self, index: u64, snapshot: &[u8]) {
        // Your code here (2D).
        crate::your_code_here((index, snapshot));
    }
}

impl Raft {
    /// Only for suppressing deadcode warnings.
    #[doc(hidden)]
    pub fn __suppress_deadcode(&mut self) {
        let _ = self.start(&0);
        let _ = self.cond_install_snapshot(0, 0, &[]);
        self.snapshot(0, &[]);
        // let _ = self.send_request_vote(0, Default::default());
        self.persist();
        let _ = &self.state;
        let _ = &self.me;
        let _ = &self.persister;
        let _ = &self.peers;
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
    // Your code here.
    raft: Arc<Mutex<Raft>>,
}

impl Node {
    /// Create a new raft service.
    pub fn new(raft: Raft) -> Node {
        // Your code here.
        let n = Node {
            raft: Arc::new(Mutex::new(raft)),
        };

        n.run_worker();
        n.run_tick();

        n
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
        debug!("call node start");
        self.raft.lock().unwrap().start(command)
    }

    fn run_tick(&self) {
        let ref_raft = self.raft.clone();
        thread::spawn(move || loop {
            {
                ref_raft.lock().unwrap().tick();
            }
            sleep(Duration::from_millis(100));
        });
    }

    fn run_worker(&self) {
        let ref_raft = self.raft.clone();
        let rx = self.raft.lock().unwrap().rx.take().unwrap();
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(m) => match m {
                    Ok(msg) => ref_raft.lock().unwrap().step(msg, None),
                    Err(e) => {
                        debug!("msg received {}", e)
                    }
                },
                Err(e) => debug!("channel error {}", e),
            };
        });
    }

    fn update_config(&mut self, election_timeout: Option<usize>, heartbeat_timeout: Option<usize>) {
        self.raft
            .lock()
            .unwrap()
            .update_config(election_timeout, heartbeat_timeout);
    }
    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        // Your code here.
        // Example:
        // self.raft.term
        self.raft.lock().unwrap().state.term
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        // Your code here.
        // Example:
        // self.raft.leader_id == self.id
        self.raft.lock().unwrap().state.is_leader
    }

    pub fn state_type(&self) -> StateType {
        self.raft.lock().unwrap().state.state_type
    }

    /// The current state of this peer.
    pub fn get_state(&self) -> State {
        self.raft.lock().unwrap().state.clone()
    }

    pub fn get_raft(&self) -> Arc<Mutex<Raft>> {
        self.raft.clone()
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
        // Your code here, if desired.
    }

    /// A service wants to switch to snapshot.  
    ///
    /// Only do so if Raft hasn't have more recent info since it communicate
    /// the snapshot on `apply_ch`.
    pub fn cond_install_snapshot(
        &self,
        last_included_term: u64,
        last_included_index: u64,
        snapshot: &[u8],
    ) -> bool {
        // Your code here.
        // Example:
        // self.raft.cond_install_snapshot(last_included_term, last_included_index, snapshot)
        crate::your_code_here((last_included_term, last_included_index, snapshot));
    }

    /// The service says it has created a snapshot that has all info up to and
    /// including index. This means the service no longer needs the log through
    /// (and including) that index. Raft should now trim its log as much as
    /// possible.
    pub fn snapshot(&self, index: u64, snapshot: &[u8]) {
        // Your code here.
        // Example:
        // self.raft.snapshot(index, snapshot)
        crate::your_code_here((index, snapshot));
    }
}

#[async_trait::async_trait]
impl RaftService for Node {
    // example RequestVote RPC handler.
    //
    // CAVEATS: Please avoid locking or sleeping here, it may jam the network.
    async fn request_vote(&self, args: RequestVoteArgs) -> labrpc::Result<RequestVoteReply> {
        // Your code here (2A, 2B).
        let (tx, rx) = mpsc::channel();

        {
            let mut raft = self.raft.lock().unwrap();
            debug!(
                "N{} -> N{} receiving request_vote {:?}",
                args.candidate_id, raft.me, args
            );
            raft.step(
                Message {
                    to: None,
                    from: None,
                    message: ProtoMessage::RequestVoteArgs(args),
                },
                Some(tx),
            );
        }

        match rx.recv().unwrap() {
            ProtoMessage::RequestVoteReply(reply) => Ok(reply),
            _ => Err(labrpc::Error::Other("unknown response".into())),
        }
    }

    async fn append_entries(&self, args: AppendEntriesArgs) -> labrpc::Result<AppendEntriesReply> {
        let (tx, rx) = mpsc::channel();

        {
            let mut raft = self.raft.lock().unwrap();
            debug!("N{} -> N{} append_entries", args.leader_id, raft.me);
            raft.step(
                Message {
                    to: None,
                    from: None,
                    message: ProtoMessage::AppendEntriesArgs(args),
                },
                Some(tx),
            );
        }

        match rx.recv().unwrap() {
            ProtoMessage::AppendEntriesReply(reply) => Ok(reply),
            _ => Err(labrpc::Error::Other("unknown response".into())),
        }
    }

    async fn heartbeat(&self, args: HeartbeatArgs) -> labrpc::Result<HeartbeatReply> {
        let (tx, rx) = mpsc::channel();

        {
            let mut raft = self.raft.lock().unwrap();
            debug!("N{} -> N{} heartbeat", args.leader_id, raft.me);
            raft.step(
                Message {
                    to: None,
                    from: None,
                    message: ProtoMessage::HeartbeatArgs(args),
                },
                Some(tx),
            );
        }

        match rx.recv().unwrap() {
            ProtoMessage::HeartbeatReply(reply) => Ok(reply),
            _ => Err(labrpc::Error::Other("unknown response".into())),
        }
    }
}
