use std::sync::{mpsc, MutexGuard};

use crate::raft::config::Config;
use crate::raft::*;

fn update_cluster_raft_config(
    c: &Config,
    election_timeout: Option<usize>,
    heartbeat_timeout: Option<usize>,
) {
    for raft in c.rafts.lock().unwrap().iter() {
        raft.as_ref()
            .unwrap()
            .get_raft()
            .lock()
            .unwrap()
            .update_config(election_timeout, heartbeat_timeout);
    }
}

#[test]
fn test_vote_follower_2aa() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2A): test_vote_follower_2aa");

    let nodes = cfg.rafts.lock().unwrap();

    let node0 = nodes[0].as_ref().unwrap();

    let state = node0.get_state();

    // truns to follower
    node0
        .raft
        .lock()
        .unwrap()
        .become_follower(state.term, 2.into());

    let new_term = state.term + 1;
    let msg = Message {
        from: Some(2),
        to: Some(0),
        message: ProtoMessage::RequestVoteArgs(RequestVoteArgs {
            term: new_term,
            candidate_id: 2,
        }),
    };

    let (tx, rx) = mpsc::channel::<ProtoMessage>();

    node0.raft.lock().unwrap().step(msg, Some(tx));

    let resp = rx.recv().unwrap();

    match resp {
        ProtoMessage::RequestVoteReply(reply) => {
            assert_eq!(new_term, reply.term);
            assert!(reply.vote_granted);
        }
        _ => panic!("unexpected resp type"),
    }
}

const TYPES: [StateType; 3] = [StateType::Follower, StateType::Candidate, StateType::Leader];

fn new_test_raft(servers: usize) -> Config {
    Config::new(servers)
}

#[test]
fn test_vote_from_any_state_2aa() {
    for t in TYPES {
        let servers = 3;
        let mut cfg = Config::new(servers);

        cfg.begin("Test (2A): test_vote_from_any_state_2aa");

        let nodes = cfg.rafts.lock().unwrap();

        let node0 = nodes[0].as_ref().unwrap();

        let state = node0.get_state();

        match t {
            StateType::Follower => {
                node0
                    .raft
                    .lock()
                    .unwrap()
                    .become_follower(state.term, 2.into());
            }
            StateType::Candidate => node0.raft.lock().unwrap().become_candidate(),
            StateType::Leader => {
                node0.raft.lock().unwrap().become_candidate();
                node0.raft.lock().unwrap().become_leader();
            }
        }

        let state = node0.get_state();

        // truns to follower

        let new_term = state.term + 1;
        let msg = Message {
            from: Some(2),
            to: Some(0),
            message: ProtoMessage::RequestVoteArgs(RequestVoteArgs {
                term: new_term,
                candidate_id: 2,
            }),
        };
        let (tx, rx) = mpsc::channel::<ProtoMessage>();

        node0.raft.lock().unwrap().step(msg, Some(tx));

        let resp = rx.recv().unwrap();

        match resp {
            ProtoMessage::RequestVoteReply(reply) => {
                assert_eq!(new_term, reply.term);
                assert!(reply.vote_granted);
            }
            _ => panic!("unexpected resp type"),
        }
    }
}

#[test]
/// TestStartAsFollower tests that when servers start up, they begin as followers.
/// Reference: section 5.2
fn test_start_at_follower_2aa() {
    let mut cfg = new_test_raft(3);
    cfg.begin("Test(2AA): test_start_at_follower_2aa");

    let nodes = cfg.rafts.lock().unwrap();

    let node0 = nodes[0].as_ref().unwrap();

    let state = node0.get_state();

    assert_eq!(StateType::Follower, state.state_type)
}

#[test]
/// test_candidate_wins tests that if candidate receives votes from a majority of the servers in the full cluster for the same term.
/// Once a candidate wins an election, it becomes leader.
/// It then sends heartbeat messages to all of the other servers to establish its authority and prevent new elections.
/// Reference: section 5.2
fn test_candidate_wins() {
    let mut cfg = new_test_raft(3);
    cfg.begin("Test(2AA): test_leadear_bcastbeat");
    let nodes = cfg.rafts.lock().unwrap();

    let node0 = nodes[0].as_ref().unwrap();

    node0.get_raft().lock().unwrap().become_candidate();

    let state = node0.get_state();

    // clear request votes msgs
    raft_wrapper(node0, |f| {
        f.msgs.lock().unwrap().clear();
    });

    raft_wrapper(node0, |f| {
        f.step(
            Message {
                from: 1.into(),
                to: 0.into(),
                message: ProtoMessage::RequestVoteReply(RequestVoteReply {
                    term: state.term,
                    vote_granted: true,
                }),
            },
            None,
        );

        f.step(
            Message {
                from: 2.into(),
                to: 0.into(),
                message: ProtoMessage::RequestVoteReply(RequestVoteReply {
                    term: state.term,
                    vote_granted: true,
                }),
            },
            None,
        );
    });

    for _i in 0..1 {
        node0.get_raft().lock().unwrap().tick();
    }

    let mut msgs = clone_raft_msgs(node0);
    msgs.sort_by_key(|m| m.to);

    let expected: Vec<Message> = Vec::from([
        Message {
            to: Some(1),
            from: Some(0),
            message: ProtoMessage::HeartbeatArgs(HeartbeatArgs {
                term: state.term,
                leader_id: 0,
            }),
        },
        Message {
            to: Some(2),
            from: Some(0),
            message: ProtoMessage::HeartbeatArgs(HeartbeatArgs {
                term: state.term,
                leader_id: 0,
            }),
        },
    ]);

    assert_eq!(expected, msgs);

    let state = node0.get_state();

    assert_eq!(StateType::Leader, state.state_type);
}

#[test]
/// TestLeaderBcastBeat tests that if the leader receives a heartbeat tick,
/// it will send a MessageType_MsgHeartbeat with m.Index = 0, m.LogTerm=0 and empty entries
/// as heartbeat to all followers.
//. Reference: section 5.2
fn test_leadear_bcastbeat() {
    let mut cfg = new_test_raft(3);
    cfg.begin("Test(2AA): test_leadear_bcastbeat");
    let nodes = cfg.rafts.lock().unwrap();

    let node0 = nodes[0].as_ref().unwrap();

    node0.get_raft().lock().unwrap().become_candidate();
    node0.get_raft().lock().unwrap().become_leader();

    let state = node0.get_state();

    // TODO(weny): sends msg propose to leader
    for _i in 0..1 {
        node0.get_raft().lock().unwrap().tick();
    }

    let mut msgs = clone_raft_msgs(node0);

    msgs.sort_by_key(|m| m.to);

    let expected: Vec<Message> = Vec::from([
        Message {
            to: Some(1),
            from: Some(0),
            message: ProtoMessage::HeartbeatArgs(HeartbeatArgs {
                term: state.term,
                leader_id: 0,
            }),
        },
        Message {
            to: Some(2),
            from: Some(0),
            message: ProtoMessage::HeartbeatArgs(HeartbeatArgs {
                term: state.term,
                leader_id: 0,
            }),
        },
    ]);

    assert_eq!(expected, msgs)
}

fn clone_raft_msgs(node0: &Node) -> Vec<Message> {
    node0
        .get_raft()
        .lock()
        .unwrap()
        .msgs
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .collect()
}

fn raft_wrapper<F>(node0: &Node, f: F)
where
    F: Fn(&mut MutexGuard<Raft>),
{
    let raft = node0.get_raft();
    let mut guard = raft.lock().unwrap();
    f(&mut guard);
}

#[test]
fn test_follower_start_election_2aa() {
    test_nonleader_start_election(StateType::Follower)
}

#[test]
fn test_candidate_start_election_2aa() {
    test_nonleader_start_election(StateType::Candidate)
}

// testNonleaderStartElection tests that if a follower receives no communication
// over election timeout, it begins an election to choose a new leader. It
// increments its current term and transitions to candidate state. It then
// votes for itself and issues RequestVote RPCs in parallel to each of the
// other servers in the cluster.
// Reference: section 5.2
// Also if a candidate fails to obtain a majority, it will time out and
// start a new election by incrementing its term and initiating another
// round of RequestVote RPCs.
// Reference: section 5.2
fn test_nonleader_start_election(state_type: StateType) {
    let mut cfg = new_test_raft(3);
    cfg.begin("Test(2AA): test_leadear_bcastbeat");
    let nodes = cfg.rafts.lock().unwrap();

    let node0 = nodes[0].as_ref().unwrap();

    let et = 10;

    match state_type {
        StateType::Follower => node0
            .get_raft()
            .lock()
            .unwrap()
            .become_follower(1, 2.into()),
        StateType::Candidate => {
            node0.get_raft().lock().unwrap().become_candidate();
        }
        _ => {}
    }

    for _i in 0..2 * et {
        node0.get_raft().lock().unwrap().tick();
    }

    // the state after election
    let state = node0.get_state();

    assert_eq!(2, state.term);
    assert_eq!(StateType::Candidate, state.state_type);
    // votes to self
    assert_eq!(Some(0), state.voted_for);
    assert!(state.votes.get(&0).unwrap());

    // request votes msgs
    let mut msgs = clone_raft_msgs(node0);

    msgs.sort_by_key(|m| m.to);

    let expected: Vec<Message> = Vec::from([
        Message {
            to: Some(1),
            from: Some(0),
            message: ProtoMessage::RequestVoteArgs(RequestVoteArgs {
                term: state.term,
                candidate_id: 0,
            }),
        },
        Message {
            to: Some(2),
            from: Some(0),
            message: ProtoMessage::RequestVoteArgs(RequestVoteArgs {
                term: state.term,
                candidate_id: 0,
            }),
        },
    ]);

    assert_eq!(expected, msgs)
}

#[test]
// TestFollowerVote tests that each follower will vote for at most one
// candidate in a given term, on a first-come-first-served basis.
// Reference: section 5.2
fn test_follower_vote2_aa() {
    struct Test {
        pub vote: Option<u64>,
        pub nvote: u64,
        pub wreject: bool,
    }

    let tests = Vec::from([
        Test {
            vote: Some(0),
            nvote: 0,
            wreject: false,
        },
        Test {
            vote: None,
            nvote: 1,
            wreject: false,
        },
        Test {
            vote: Some(0),
            nvote: 0,
            wreject: false,
        },
        Test {
            vote: Some(1),
            nvote: 1,
            wreject: false,
        },
        Test {
            vote: Some(0),
            nvote: 1,
            wreject: true,
        },
        Test {
            vote: Some(1),
            nvote: 0,
            wreject: true,
        },
    ]);

    for test in tests {
        let mut cfg = new_test_raft(3);
        cfg.begin("Test(2AA): test_follower_vote2_aa");
        let nodes = cfg.rafts.lock().unwrap();
        let node0 = nodes[0].as_ref().unwrap();

        raft_wrapper(node0, |f| {
            f.state.term = 1;
            f.state.voted_for = test.vote;

            let (tx, rx) = mpsc::channel();
            f.step(
                Message {
                    from: Some(test.nvote),
                    to: Some(0),
                    message: ProtoMessage::RequestVoteArgs(RequestVoteArgs {
                        term: 1,
                        candidate_id: test.nvote,
                    }),
                },
                Some(tx),
            );

            let msgs = rx.recv().unwrap();
            let expected = ProtoMessage::RequestVoteReply(RequestVoteReply {
                term: 1,
                vote_granted: !test.wreject,
            });

            assert_eq!(msgs, expected);
        })
    }
}

#[test]
// TestCandidateFallback tests that while waiting for votes,
// if a candidate receives an AppendEntries RPC from another server claiming
// to be leader whose term is at least as large as the candidate's current term,
// it recognizes the leader as legitimate and returns to follower state.
// Reference: section 5.2
fn test_candiate_fallback_2aa() {
    struct Test {
        msg: ProtoMessage,
        term: u64,
    }
    let tests = [
        Test {
            msg: ProtoMessage::AppendEntriesArgs(AppendEntriesArgs {
                term: 1,
                leader_id: 2,
                prev_log_term: 0,
                entries: Vec::new(),
                leader_commit: 0,
            }),
            term: 1,
        },
        Test {
            msg: ProtoMessage::AppendEntriesArgs(AppendEntriesArgs {
                term: 2,
                leader_id: 2,
                prev_log_term: 0,
                entries: Vec::new(),
                leader_commit: 0,
            }),
            term: 2,
        },
    ];

    for tt in tests {
        let mut cfg = new_test_raft(3);
        cfg.begin("Test(2AA): test_candiate_fallback_2aa");
        let nodes = cfg.rafts.lock().unwrap();
        let node0 = nodes[0].as_ref().unwrap();

        raft_wrapper(node0, |f| {
            f.step(ProtoMessage::MsgHup.into(), None);
        });

        raft_wrapper(node0, |f| {
            let (tx, rx) = mpsc::channel();
            f.step(tt.msg.clone().into(), Some(tx));
            // blocks
            let _ = rx.recv();
        });

        let state = node0.get_state();

        assert_eq!(StateType::Follower, state.state_type);
        assert_eq!(tt.term, state.term);
    }
}

// skips election timeout randomized tests
