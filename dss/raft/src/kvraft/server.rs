use std::sync::{Arc, Mutex, MutexGuard};

use futures::sync::mpsc::{unbounded, UnboundedReceiver};
use uuid::Uuid;

use labrpc::RpcFuture;

use crate::proto::kvraftpb::*;
use crate::raft;
use crate::raft::ApplyMsg;
use failure::Fail;
use futures::sync::oneshot::Sender;
use futures::{Future, Stream};
use labcodec::Message;
use std::collections::{BTreeMap, HashMap, HashSet};

#[allow(dead_code)]
enum KvCommand {
    Get {
        id: Uuid,
    },
    Put {
        id: Uuid,
        key: String,
        value: String,
    },
    Append {
        id: Uuid,
        key: String,
        value: String,
    },
}

fn build_uuid(origin: &[u8]) -> Uuid {
    Uuid::from_slice(origin).expect("Failed to parse uuid from request.")
}

impl KvCommand {
    fn from_bytes(proto: &[u8]) -> Option<Self> {
        if let Ok(putting) = labcodec::decode::<PutAppendRequest>(proto) {
            return Some(KvCommand::from_put_append(putting));
        }
        if let Ok(getting) = labcodec::decode::<GetRequest>(proto) {
            return Some(KvCommand::from_get(getting));
        }
        warn!("failed to parse proto buffer message {:?}.", proto);
        None
    }

    fn from_put_append(request: PutAppendRequest) -> Self {
        match Op::from_i32(request.op).unwrap_or(Op::Unknown) {
            Op::Unknown => panic!("unknown op detached."),
            Op::Put => KvCommand::Put {
                id: build_uuid(request.id.as_slice()),
                key: request.key,
                value: request.value,
            },
            Op::Append => KvCommand::Append {
                id: build_uuid(request.id.as_slice()),
                key: request.key,
                value: request.value,
            },
        }
    }

    fn from_get(request: GetRequest) -> Self {
        KvCommand::Get {
            id: build_uuid(request.id.as_slice()),
        }
    }

    fn get_id(&self) -> &Uuid {
        match self {
            KvCommand::Get { id } => id,
            KvCommand::Put { id, .. } => id,
            KvCommand::Append { id, .. } => id,
        }
    }
}

#[derive(Fail, Debug)]
enum KvError {
    // TODO: 加上 cause
    #[fail(display = "Raft response error.")]
    Raft(raft::errors::Error),
    #[fail(display = "Current node isn't leader.")]
    NotLeader,
}

type Result<T> = std::result::Result<T, KvError>;

#[derive(Clone)]
struct KvStateMachine {
    index: usize,
    pub state: Arc<Mutex<BTreeMap<String, String>>>,
    pending_commands: Arc<Mutex<HashSet<Uuid>>>,
    waiting_channels: Arc<Mutex<HashMap<u64, Sender<()>>>>,
    raft: raft::Node,
}

impl KvStateMachine {
    fn new(apply_ch: UnboundedReceiver<ApplyMsg>, raft: raft::Node) -> Self {
        let state = Arc::new(Mutex::default());
        let pending_commands = Arc::new(Mutex::default());
        let waiting_channels = Arc::new(Mutex::default());
        // spawn the handler.
        std::thread::spawn({
            let pending_commands = pending_commands.clone();
            let state = state.clone();
            let waiting_channels = waiting_channels.clone();
            move || {
                apply_ch
                    .for_each(|message| {
                        let mut notifier = waiting_channels.lock().unwrap();
                        if let Some(sender) =
                            HashMap::remove(&mut *notifier, &message.command_index)
                        {
                            Sender::send(sender, ()).unwrap_or_else(|_| {
                                warn!(
                                    "Message notifier doesn't send rightly. \
                                     Maybe raft commits this too fast."
                                );
                            });
                        }
                        let command = KvCommand::from_bytes(message.command.as_slice());
                        if command.is_none() {
                            panic!("Invalid message received.")
                        }
                        let cmd = command.unwrap();
                        let id = *cmd.get_id();
                        match cmd {
                            KvCommand::Get { .. } => {}
                            KvCommand::Put { key, value, .. } => {
                                let mut state: MutexGuard<BTreeMap<String, String>> =
                                    state.lock().unwrap();
                                let map = &mut *state;
                                map.insert(key, value);
                            }
                            KvCommand::Append { key, value, .. } => {
                                let mut state = state.lock().unwrap();
                                if let Some(val) = state.get_mut(&key) {
                                    val.push_str(value.as_str());
                                }
                            }
                        }
                        let mut pending_commands = pending_commands.lock().unwrap();
                        HashSet::remove(&mut *pending_commands, &id);
                        futures::finished(())
                    })
                    .wait()
                    .expect("FSM supporting worker panicked.");
            }
        });
        KvStateMachine {
            index: 1,
            state,
            pending_commands,
            waiting_channels,
            raft,
        }
    }

    fn start(
        &self,
        cmd: &impl Message,
    ) -> Box<dyn Future<Item = Result<()>, Error = ()> + Send + 'static> {
        use crate::raft::errors::Error;
        match self.raft.start(cmd) {
            // TODO: 这里 Raft 可能会过早提交成功导致无法收到通知。
            Ok((idx, _term)) => {
                let (sx, rx) = futures::sync::oneshot::channel::<()>();
                let mut map = self.waiting_channels.lock().unwrap();
                map.insert(idx, sx);
                Box::new(
                    rx.map(Ok)
                        .map_err(|e| warn!("FSM::start received exception: {}", e)),
                )
            }
            Err(Error::NotLeader) => Box::new(futures::finished(Err(KvError::NotLeader))),
            Err(e) => Box::new(futures::finished(Err(KvError::Raft(e)))),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        let state = self.state.lock().unwrap();
        state.get(key).cloned()
    }
}

pub struct KvServer {
    pub rf: raft::Node,
    me: usize,
    // snapshot if log grows this big
    maxraftstate: Option<usize>,
    // Your definitions here.
    fsm: KvStateMachine,
}

impl KvServer {
    pub fn new(
        servers: Vec<crate::proto::raftpb::RaftClient>,
        me: usize,
        persister: Box<dyn raft::persister::Persister>,
        maxraftstate: Option<usize>,
    ) -> KvServer {
        // You may need initialization code here.

        let (tx, apply_ch) = unbounded();
        let rf = raft::Raft::new(servers, me, persister, tx);
        let node = raft::Node::new(rf);
        let fsm = KvStateMachine::new(apply_ch, node.clone());
        KvServer {
            rf: node,
            me,
            maxraftstate,
            fsm,
        }
    }
}

impl KvServer {
    /// Only for suppressing deadcode warnings.
    #[doc(hidden)]
    pub fn __suppress_deadcode(&mut self) {
        let _ = KvCommand::Get { id: Uuid::nil() };
        let _ = &self.me;
        let _ = &self.maxraftstate;
        let _ = KvCommand::from_bytes(vec![].as_slice());
        crate::your_code_here(());
    }
}

// Choose concurrency paradigm.
//
// You can either drive the kv server by the rpc framework,
//
// ```rust
// struct Node { server: Arc<Mutex<KvServer>> }
// ```
//
// or spawn a new thread runs the kv server and communicate via
// a channel.
//
// ```rust
// struct Node { sender: Sender<Msg> }
// ```
#[derive(Clone)]
pub struct Node {
    server: Arc<Mutex<KvServer>>,
}

impl Node {
    pub fn new(kv: KvServer) -> Node {
        let server = Arc::new(Mutex::new(kv));
        Node { server }
    }

    /// the tester calls Kill() when a KVServer instance won't
    /// be needed again. you are not required to do anything
    /// in Kill(), but it might be convenient to (for example)
    /// turn off debug output from this instance.
    pub fn kill(&self) {
        // Your code here, if desired.
    }

    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.get_state().term()
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        self.get_state().is_leader()
    }

    pub fn get_state(&self) -> raft::State {
        let server = self.server.lock().unwrap();
        server.rf.get_state()
    }
}

impl KvService for Node {
    // CAVEATS: Please avoid locking or sleeping here, it may jam the network.
    fn get(&self, arg: GetRequest) -> RpcFuture<GetReply> {
        let server = self.server.lock().unwrap();
        let fsm = server.fsm.clone();
        drop(server);
        let start_result = fsm.start(&arg);
        let reply_fut = start_result.map(move |result| match result {
            Err(KvError::NotLeader) => GetReply {
                wrong_leader: true,
                err: "not leader".to_owned(),
                value: "".to_owned(),
            },
            Ok(()) => GetReply {
                wrong_leader: false,
                err: "".to_owned(),
                value: fsm.get(&arg.key).unwrap_or_else(|| "".to_owned()),
            },
            Err(e) => GetReply {
                wrong_leader: false,
                err: format!("{}", e),
                value: "".to_owned(),
            },
        });
        Box::new(reply_fut.map_err(|_| panic!("fetal: failed to send rpc: failed to execute.")))
    }

    // CAVEATS: Please avoid locking or sleeping here, it may jam the network.
    fn put_append(&self, arg: PutAppendRequest) -> RpcFuture<PutAppendReply> {
        let server = self.server.lock().unwrap();
        let start_result = server.fsm.start(&arg);
        drop(server);
        let reply_fut = start_result.map(|result| match result {
            Err(KvError::NotLeader) => PutAppendReply {
                wrong_leader: true,
                err: "not leader".to_owned(),
            },
            Ok(()) => PutAppendReply {
                wrong_leader: false,
                err: "".to_owned(),
            },
            Err(e) => PutAppendReply {
                wrong_leader: false,
                err: format!("{}", e),
            },
        });
        Box::new(reply_fut.map_err(|_| panic!("fetal: failed to send rpc: failed to execute.")))
    }
}
