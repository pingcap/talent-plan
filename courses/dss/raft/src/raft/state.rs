use crate::raft::*;

impl Raft {
    pub fn become_leader(&mut self) {
        self.election_elapsed = 0;
        self.heartbeat_elapsed = 0;
        self.state.leader_id = Some(self.me as u64);
        self.state.state_type = StateType::Leader;

        info!("N{} become leader in term {}", self.me, self.state.term);
    }

    pub fn become_candidate(&mut self) {
        self.randomized_election_timeout =
            rand::thread_rng().gen_range(self.election_timeout, 2 * self.election_timeout);

        // turns to candidate
        self.state.state_type = StateType::Candidate;

        // increases term
        self.state.term += 1;

        // clears leader id
        self.state.leader_id = None;

        // votes for self
        self.state.voted_for = Some(self.me as u64);

        self.state.votes.clear();

        // updates coutners
        self.state.votes.insert(self.me as u64, true);

        // resets electionElapsed
        self.election_elapsed = 0;

        info!("N{} become Candidate in Term {}", self.me, self.state.term)
    }

    pub fn become_follower(&mut self, term: u64, leader: Option<u64>) {
        self.randomized_election_timeout =
            rand::thread_rng().gen_range(self.election_timeout, 2 * self.election_timeout);

        // turns to candidate
        self.state.state_type = StateType::Follower;

        // increases term
        self.state.term = term;

        // clears leader id
        self.state.leader_id = leader;

        // resets electionElapsed
        self.election_elapsed = 0;

        info!(
            "N{} become the Follower of N{:?} in Term {}",
            self.me, leader, term
        )
    }
}
