use crate::raft::*;

impl Raft {
    /// example code to send a RequestVote RPC to a server.
    /// server is the index of the target server in peers.
    /// expects RPC arguments in args.
    ///
    /// The labrpc package simulates a lossy network, in which servers
    /// may be unreachable, and in which requests and replies may be lost.
    /// This method sends a request and waits for a reply. If a reply arrives
    /// within a timeout interval, This method returns Ok(_); otherwise
    /// this method returns Err(_). Thus this method may not return for a while.
    /// An Err(_) return can be caused by a dead server, a live server that
    /// can't be reached, a lost request, or a lost reply.
    ///
    /// This method is guaranteed to return (perhaps after a delay) *except* if
    /// the handler function on the server side does not return.  Thus there
    /// is no need to implement your own timeouts around this method.
    ///
    /// look at the comments in ../labrpc/src/lib.rs for more details.
    pub fn send_request_vote(&mut self, server: usize, args: RequestVoteArgs) {
        self.append_message(server, ProtoMessage::RequestVoteArgs(args));
    }

    pub fn request_vote(&mut self) {
        if self.peers.len() < 2 {
            debug!(
                "request_vote: standlone server {} become leader directly",
                self.me
            );
            self.become_leader();
        } else {
            for i in 0..self.peers.len() {
                if i != self.me {
                    self.send_request_vote(
                        i,
                        RequestVoteArgs {
                            term: self.state.term,
                            candidate_id: self.me as u64,
                        },
                    );
                }
            }
        }
    }

    pub fn handle_request_vote_reply(&mut self, from: u64, m: RequestVoteReply) {
        if m.term == self.state.term {
            self.state.votes.insert(from, m.vote_granted);
            info!("handle_request_vote_reply: N{} -> N{}", from, self.me);

            if self.state.votes.len() > self.peers.len() / 2 {
                info!(
                    "handle_request_vote_reply: N{} -> N{} couting votes",
                    from, self.me
                );

                let mut agreed = 0;
                let mut declined = 0;
                for (_, &granted) in self.state.votes.iter() {
                    if granted {
                        agreed += 1;
                    } else {
                        declined += 1;
                    }
                }

                if agreed > self.peers.len() / 2 {
                    self.become_leader();
                    info!(
                        "handle_request_vote_reply: N{} quorum aggre, become leader",
                        self.me
                    );
                } else if declined > self.peers.len() / 2 {
                    self.become_follower(m.term, None);
                    info!(
                        "handle_request_vote_reply: N{} quorum aggre, become follower",
                        self.me
                    );
                }
            }
        } else if m.term > self.state.term {
            debug!(
                "handle_request_vote_reply: N{} received higher term reply",
                self.me
            );
            self.become_follower(m.term, None)
        }
    }

    pub fn handle_request_vote(&mut self, from: u64, m: RequestVoteArgs) -> ProtoMessage {
        // resets status without electionElapsed
        if m.term > self.state.term {
            debug!(
                "handle request_vote: N{:?} follows the term {} ",
                self.me, m.term
            );
            self.state.term = m.term;
            self.state.voted_for = None;
            self.state.leader_id = None;
            self.state.state_type = StateType::Follower;
        }

        if self.state.leader_id.is_some()
            || (self.state.voted_for.is_some() && self.state.voted_for.unwrap() != from)
        {
            // rejects
            debug!(
                "handle request_vote: N{:?} -> N{} rejects, due to already voted for {:?} or following leader {:?}",
                self.me, from, self.state.voted_for,self.state.leader_id
            );

            return ProtoMessage::RequestVoteReply(RequestVoteReply {
                term: self.state.term,
                vote_granted: false,
            });
        }
        self.election_elapsed = 0;
        self.state.voted_for = from.into();
        debug!("handle request_vote: N{:?} -> N{} agreed!", self.me, from);
        ProtoMessage::RequestVoteReply(RequestVoteReply {
            term: self.state.term,
            vote_granted: true,
        })
    }
}
