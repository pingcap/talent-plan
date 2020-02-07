pub mod raftpb {
    pub use self::raft::{
        add_service as add_raft_service, Client as RaftClient, Service as RaftService,
    };

    include!(concat!(env!("OUT_DIR"), "/raftpb.rs"));

    labrpc::service! {
        service raft {
            rpc request_vote(RequestVoteArgs) returns (RequestVoteReply);
            rpc append_entries(AppendEntriesArgs) returns (AppendEntriesReply);
            // Your code here if more rpc desired.
            // rpc xxx(yyy) returns (zzz)

            rpc install_snapshot(InstallSnapshotArgs) returns (InstallSnapshotReply);
        }
    }
}

pub mod kvraftpb {
    pub use self::kv::{add_service as add_kv_service, Client as KvClient, Service as KvService};

    include!(concat!(env!("OUT_DIR"), "/kvraftpb.rs"));

    labrpc::service! {
        service kv {
            rpc get(GetRequest) returns (GetReply);
            rpc put_append(PutAppendRequest) returns (PutAppendReply);

            // Your code here if more rpc desired.
            // rpc xxx(yyy) returns (zzz)
        }
    }
}
