pub mod raftpb {
    include!(concat!(env!("OUT_DIR"), "/raftpb.rs"));

    labrpc::service! {
        service raft {
            rpc request_vote(RequestVoteArgs) returns (RequestVoteReply);
            rpc append_entries(AppendEntriesArgs) returns (AppendEntriesReply);
            rpc heartbeat(HeartbeatArgs) returns (HeartbeatReply);
            // Your code here if more rpc desired.
            // rpc xxx(yyy) returns (zzz)
        }
    }
    pub use self::raft::{
        add_service as add_raft_service, Client as RaftClient, Service as RaftService,
    };
}

pub mod kvraftpb {
    include!(concat!(env!("OUT_DIR"), "/kvraftpb.rs"));

    labrpc::service! {
        service kv {
            rpc get(GetRequest) returns (GetReply);
            rpc put_append(PutAppendRequest) returns (PutAppendReply);

            // Your code here if more rpc desired.
            // rpc xxx(yyy) returns (zzz)
        }
    }
    pub use self::kv::{add_service as add_kv_service, Client as KvClient, Service as KvService};
}

use self::raftpb::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ProtoMessage {
    RequestVoteArgs(RequestVoteArgs),
    RequestVoteReply(RequestVoteReply),
    AppendEntriesArgs(AppendEntriesArgs),
    AppendEntriesReply(AppendEntriesReply),
    HeartbeatArgs(HeartbeatArgs),
    HeartbeatReply(HeartbeatReply),
    /// [`ProposeArgs`] is a local message that proposes to append data to the leader's log entries.
    ProposeArgs(ProposeArgs),
    MsgHup,  // local message
    MsgBeat, // local message
    Noop,
}
