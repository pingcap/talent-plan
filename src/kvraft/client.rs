use labrpc;
use std::fmt;

use super::service;

enum Op {
    Put(Vec<u8>, Vec<u8>),
    Append(Vec<u8>, Vec<u8>),
}

pub struct Clerk {
    pub name: String,
    servers: Vec<service::KvClient>,
    // You will have to modify this struct.
}

impl fmt::Debug for Clerk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Clerk").field("name", &self.name).finish()
    }
}

impl Clerk {
    pub fn new(name: String, servers: Vec<service::KvClient>) -> Clerk {
        // You'll have to add code here.
        Clerk { name, servers }
    }

    /// fetch the current value for a key.
    /// returns "" if the key does not exist.
    /// keeps trying forever in the face of all other errors.
    ///
    /// you can send an RPC with code like this:
    /// ok := ck.servers[i].Call("KVServer.Get", &args, &reply)
    ///
    /// the types of args and reply (including whether they are pointers)
    /// must match the declared types of the RPC handler function's
    /// arguments. and reply must be passed as a pointer.
    pub fn get(&self, key: Vec<u8>) -> Vec<u8> {
        unimplemented!()
    }

    /// shared by Put and Append.
    ///
    /// you can send an RPC with code like this:
    /// ok := ck.servers[i].Call("KVServer.PutAppend", &args, &reply)
    ///
    /// the types of args and reply (including whether they are pointers)
    /// must match the declared types of the RPC handler function's
    /// arguments. and reply must be passed as a pointer.
    fn put_append(&self, op: Op) {
        // You will have to modify this function.
        unimplemented!()
    }

    pub fn put(&self, key: Vec<u8>, value: Vec<u8>) {
        self.put_append(Op::Put(key, value))
    }

    pub fn append(&self, key: Vec<u8>, value: Vec<u8>) {
        self.put_append(Op::Append(key, value))
    }
}
