use crate::raft::*;

impl Raft {
    pub fn append_message(&mut self, to: usize, message: ProtoMessage) {
        self.msgs.lock().unwrap().push_back(Message {
            from: Some(self.me as u64),
            to: Some(to as u64),
            message,
        });
    }
}
