use std::sync::{Arc, Mutex, MutexGuard};

use futures::sync::mpsc::{unbounded, UnboundedReceiver};
use uuid::Uuid;

use labrpc::RpcFuture;

use crate::kvraft::server::KvError::FailToCommit;
use crate::proto::kvraftpb::*;
use crate::raft;
use crate::raft::ApplyMsg;
use failure::Fail;
use futures::sync::oneshot::Sender;
use futures::{Future, Stream};
use labcodec::Message;
use std::collections::{BTreeMap, HashSet};

#[allow(dead_code)]
#[derive(Clone, Debug)]
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

    fn get_id(&self) -> Uuid {
        *match self {
            KvCommand::Get { id } => id,
            KvCommand::Put { id, .. } => id,
            KvCommand::Append { id, .. } => id,
        }
    }
}

#[derive(Fail, Debug)]
// TODO: 移除这个 dead_code。
#[allow(dead_code)]
enum KvError {
    // TODO: 加上 cause
    #[fail(display = "Raft response error.")]
    Raft(raft::errors::Error),
    #[fail(display = "Current node isn't leader.")]
    NotLeader,
    #[fail(display = "replicated command.")]
    Replicated,
    #[fail(display = "The command failed to commit.")]
    FailToCommit,
}

type Result<T> = std::result::Result<T, KvError>;

#[derive(Clone)]
struct KvStateMachine {
    state: Arc<Mutex<BTreeMap<String, String>>>,
    success_commands: Arc<Mutex<HashSet<Uuid>>>,
    waiting_channels: Arc<Mutex<BTreeMap<u64, Sender<Uuid>>>>,
    raft: raft::Node,
    name: String,
}

trait Identifiable {
    fn get_id(&self) -> Uuid;
}

impl Identifiable for PutAppendRequest {
    fn get_id(&self) -> Uuid {
        Uuid::from_slice(self.id.as_slice()).unwrap()
    }
}

impl Identifiable for GetRequest {
    fn get_id(&self) -> Uuid {
        Uuid::from_slice(self.id.as_slice()).unwrap()
    }
}

impl KvStateMachine {
    fn new(apply_ch: UnboundedReceiver<ApplyMsg>, raft: raft::Node, name: String) -> Self {
        let state = Arc::new(Mutex::default());
        let success_commands = Arc::new(Mutex::new(HashSet::new()));
        let waiting_channels = Arc::new(Mutex::new(BTreeMap::new()));
        // spawn the handler.
        std::thread::spawn({
            let success_commands = success_commands.clone();
            let state = state.clone();
            let waiting_channels = waiting_channels.clone();
            let name = name.clone();
            info!("FSM worker for {} start!", name);
            move || {
                apply_ch
                    .wait()
                    .map(std::result::Result::unwrap)
                    .for_each(|message| {
                        info!("{}: message[{}] received...", name, message.command_index);

                        let command = KvCommand::from_bytes(message.command.as_slice());
                        if command.is_none() {
                            panic!("Invalid message received.")
                        }
                        let cmd = command.unwrap();
                        debug!("request: {:?}", cmd.get_id());

                        let mut notifier = waiting_channels.lock().unwrap();
                        if let Some(sender) = notifier.remove(&message.command_index) {
                            Sender::send(sender, cmd.get_id()).unwrap_or_else(|_| {
                                warn!(
                                    "Message notifier doesn't send rightly. \
                                     Maybe raft commits this too fast."
                                );
                            });
                        }

                        let id = cmd.get_id();
                        let mut history = success_commands.lock().unwrap();
                        if history.contains(&id) {
                            warn!(
                                "{}: Replicated ID: {} get, the command {:?} won't be processed.",
                                name, id, cmd
                            );
                            return;
                        }
                        history.insert(id);
                        drop(history);

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
                    })
            }
        });
        KvStateMachine {
            state,
            success_commands,
            waiting_channels,
            raft,
            name,
        }
    }

    fn start(
        &self,
        cmd: &(impl Message + Identifiable),
    ) -> Box<dyn Future<Item = Result<()>, Error = ()> + Send + 'static> {
        use crate::raft::errors::Error;

        match self.raft.start(cmd) {
            // TODO: 这里 Raft 可能会过早提交成功导致无法收到通知。
            Ok((idx, _term)) => {
                let (sx, rx) = futures::sync::oneshot::channel();
                let mut map = self.waiting_channels.lock().unwrap();
                map.insert(idx, sx);
                let cmd_id = cmd.get_id();
                info!("{}: started message[{}]", self.name, idx);
                Box::new(
                    rx.map(move |id| {
                        if id == cmd_id {
                            Ok(())
                        } else {
                            Err(FailToCommit)
                        }
                    })
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

    fn contains_command(&self, id: Uuid) -> bool {
        let commands = self.success_commands.lock().unwrap();
        commands.contains(&id)
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
        let fsm = KvStateMachine::new(apply_ch, node.clone(), format!("[{}]", me));
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
        let cmd_id = Uuid::from_slice(arg.id.as_slice()).expect("fetal: bad command id.");
        if server.fsm.contains_command(cmd_id) {
            return Box::new(futures::finished(PutAppendReply {
                wrong_leader: false,
                err: "".to_owned(),
            }));
        }
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
