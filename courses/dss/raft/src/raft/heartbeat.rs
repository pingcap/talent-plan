use crate::raft::*;

impl Raft {
    pub fn update_state_from_heartbeat(&mut self, _from: u64, args: &HeartbeatArgs) {
        self.election_elapsed = 0;

        // for first receving heartbeat
        if args.term > self.state.term {
            self.become_follower(args.term, args.leader_id.into());
        }
    }

    pub fn bcastbeat(&mut self) {
        for i in 0..self.peers.len() {
            if i != self.me {
                self.send_heartbeat(i);
            }
        }
    }

    pub fn send_heartbeat(&mut self, to: usize) {
        self.append_message(
            to,
            ProtoMessage::HeartbeatArgs(HeartbeatArgs {
                term: self.state.term,
                leader_id: self.me as u64,
            }),
        )
    }

    pub fn handle_heartbeat(&mut self, _from: u64, _m: HeartbeatArgs) -> ProtoMessage {
        ProtoMessage::HeartbeatReply(HeartbeatReply {
            term: self.state.term,
            commited_index: 0, //TODO(weny):add  raftlog.commited_index
        })
    }

    pub fn handle_heartbeat_reply(&mut self, _m: HeartbeatReply) {}
}
