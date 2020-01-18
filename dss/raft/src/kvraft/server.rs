use std::sync::{Arc, Mutex};

use futures::sync::mpsc::{unbounded, UnboundedReceiver};
use uuid::Uuid;

use labrpc::RpcFuture;

use crate::proto::kvraftpb::*;
use crate::raft;
use crate::raft::ApplyMsg;

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
}

pub struct KvServer {
    pub rf: raft::Node,
    me: usize,
    // snapshot if log grows this big
    maxraftstate: Option<usize>,
    // Your definitions here.
    apply_ch: UnboundedReceiver<ApplyMsg>,
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
        KvServer {
            rf: raft::Node::new(rf),
            me,
            maxraftstate,
            apply_ch,
        }
    }
}

impl KvServer {
    /// Only for suppressing deadcode warnings.
    #[doc(hidden)]
    pub fn __suppress_deadcode(&mut self) {
        let _ = KvCommand::Get { id: Uuid::nil() };
        let _ = &self.apply_ch;
        let _ = &self.me;
        let _ = &self.maxraftstate;
        let _ = KvCommand::from_bytes(vec![].as_slice());
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
        // Your code here.
        crate::your_code_here(arg)
    }

    // CAVEATS: Please avoid locking or sleeping here, it may jam the network.
    fn put_append(&self, arg: PutAppendRequest) -> RpcFuture<PutAppendReply> {
        // Your code here.
        crate::your_code_here(arg)
    }
}
