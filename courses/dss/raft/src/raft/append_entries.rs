use super::*;

impl Raft {
    pub fn handle_append_entries(&mut self, _args: AppendEntriesArgs) -> ProtoMessage {
        ProtoMessage::AppendEntriesReply(AppendEntriesReply {
            term: self.state.term,
            success: true,
        })
    }
}
