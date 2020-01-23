use std::fmt;

use crate::proto::kvraftpb::*;
use futures::Future;
use labrpc::Error;
use std::cell::Cell;
use std::time::Duration;
use uuid::Uuid;

enum Op {
    Put(String, String),
    Append(String, String),
}

pub struct Clerk {
    pub name: String,
    pub servers: Vec<KvClient>,
    // You will have to modify this struct.
    leader: Cell<Option<usize>>,
}

impl Into<PutAppendRequest> for Op {
    fn into(self) -> PutAppendRequest {
        match self {
            Op::Put(key, value) => PutAppendRequest {
                id: Clerk::new_id(),
                key,
                value,
                op: 1,
            },
            Op::Append(key, value) => PutAppendRequest {
                id: Clerk::new_id(),
                key,
                value,
                op: 2,
            },
        }
    }
}

impl fmt::Debug for Clerk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Clerk").field("name", &self.name).finish()
    }
}

impl Clerk {
    pub fn new(name: String, servers: Vec<KvClient>) -> Clerk {
        // You'll have to add code here.
        // Clerk { name, servers }
        Clerk {
            name,
            servers,
            leader: Cell::new(None),
        }
    }

    fn new_id() -> Vec<u8> {
        let id = Uuid::new_v4();
        id.as_bytes().to_vec()
    }

    fn try_send_to_current_leader<R>(
        &self,
        send: impl Fn(&KvClient) -> R,
        is_leader: impl Fn(&R) -> bool,
    ) -> Option<R> {
        if let Some(leader) = self.leader.get() {
            let message = send(&self.servers[leader]);
            return if !is_leader(&message) {
                // leadership changed.
                self.leader.set(None);
                None
            } else {
                Some(message)
            };
        }
        None
    }

    fn check_leader_and_send<R>(
        &self,
        send: impl Fn(&KvClient) -> R,
        is_leader: impl Fn(&R) -> bool,
    ) -> R {
        loop {
            for (i, client) in self.servers.iter().enumerate() {
                let result = send(client);
                if is_leader(&result) {
                    self.leader.set(Some(i));
                    return result;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn request<R>(&self, send: impl Fn(&KvClient) -> R, is_leader: impl Fn(&R) -> bool) -> R {
        // first: send to current leader.
        let try_result = self.try_send_to_current_leader(&send, &is_leader);
        if let Some(message) = try_result {
            return message;
        }

        // when leadership changed... or we cannot detect that who is the leader...
        assert!(
            self.leader.get().is_none(),
            "We tried to find new leader even there is a leader available."
        );
        self.check_leader_and_send(&send, &is_leader)
    }

    /// fetch the current value for a key.
    /// returns "" if the key does not exist.
    /// keeps trying forever in the face of all other errors.
    //
    // you can send an RPC with code like this:
    // if let Some(reply) = self.servers[i].get(args).wait() { /* do something */ }
    pub fn get(&self, key: String) -> String {
        let args = GetRequest {
            id: Self::new_id(),
            key,
        };

        let send = |client: &KvClient| client.get(&args).wait();
        let is_leader = |reply: &Result<GetReply, Error>| match reply {
            Err(_) => false,
            Ok(message) if message.wrong_leader => false,
            _ => true,
        };

        loop {
            match self.request(&send, &is_leader) {
                Ok(message) => return message.value,
                Err(_) => continue,
            }
        }
    }

    /// shared by Put and Append.
    //
    // you can send an RPC with code like this:
    // let reply = self.servers[i].put_append(args).unwrap();
    fn put_append(&self, op: Op) {
        // You will have to modify this function.
        let args: PutAppendRequest = op.into();
        let send = |client: &KvClient| client.put_append(&args).wait();
        let is_leader = |reply: &Result<PutAppendReply, Error>| match reply {
            Err(_) => false,
            Ok(message) if message.wrong_leader => false,
            _ => true,
        };

        loop {
            match self.request(&send, &is_leader) {
                Ok(_) => return,
                Err(_) => continue,
            }
        }
    }

    pub fn put(&self, key: String, value: String) {
        self.put_append(Op::Put(key, value))
    }

    pub fn append(&self, key: String, value: String) {
        self.put_append(Op::Append(key, value))
    }
}
