use raft::config::{Config, Entry};
use std::thread;
use std::time::Duration;

/// The tester generously allows solutions to complete elections in one second
/// (much more than the paper's range of timeouts).
const RAFT_ELECTION_TIMEOUT: Duration = Duration::from_millis(1000);

#[test]
fn test_initial_election_2a() {
    let servers = 3;
    let mut cfg = Config::new(servers, false);

    cfg.begin("Test (2A): initial election");

    // is a leader elected?
    cfg.check_one_leader();

    // sleep a bit to avoid racing with followers learning of the
    // election, then check that all peers agree on the term.
    thread::sleep(Duration::from_millis(50));
    let term1 = cfg.check_terms();

    // does the leader+term stay the same if there is no network failure?
    thread::sleep(2 * RAFT_ELECTION_TIMEOUT);
    let term2 = cfg.check_terms();
    if term1 != term2 {
        warn!("warning: term changed even though there were no failures")
    }

    // there should still be a leader.
    cfg.check_one_leader();

    cfg.end();
}
