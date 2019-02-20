labrpc::service! {
    service kv {
        rpc get(GetRequest) returns (GetReply);
        rpc put_append(PutAppendRequest) returns (PutAppendReply);
    }
}
pub use self::kv::{add_service as add_kv_service, Client as KvClient, Service as KvService};

/// Put or Append
#[derive(Clone, PartialEq, Message)]
pub struct PutAppendRequest {
    #[prost(string, tag = "1")]
    pub key: String,
    #[prost(string, tag = "2")]
    pub value: String,
    // "Put" or "Append"
    #[prost(enumeration = "Op", tag = "3")]
    pub op: i32,
    // You'll have to add definitions here.
}

#[derive(Clone, PartialEq, Message)]
pub struct PutAppendReply {
    #[prost(bool, tag = "1")]
    pub wrong_leader: bool,
    #[prost(string, tag = "2")]
    pub err: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetRequest {
    #[prost(string, tag = "1")]
    pub key: String,
    // You'll have to add definitions here.
}

#[derive(Clone, PartialEq, Message)]
pub struct GetReply {
    #[prost(bool, tag = "1")]
    pub wrong_leader: bool,
    #[prost(string, tag = "2")]
    pub err: String,
    #[prost(string, tag = "3")]
    pub value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Enumeration)]
pub enum Op {
    Unknown = 0,
    Put = 1,
    Append = 2,
}
