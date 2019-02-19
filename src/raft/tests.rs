use raft::config::{Config, Entry};
use std::sync::mpsc::channel;
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

#[test]
fn test_reelection_2a() {
    let servers = 3;
    let mut cfg = Config::new(servers, false);
    cfg.begin("Test (2A): election after network failure");

    let leader1 = cfg.check_one_leader();
    // if the leader disconnects, a new one should be elected.
    cfg.disconnect(leader1);
    cfg.check_one_leader();

    // if the old leader rejoins, that shouldn't
    // disturb the new leader.
    cfg.connect(leader1);
    let leader2 = cfg.check_one_leader();

    // if there's no quorum, no leader should
    // be elected.
    cfg.disconnect(leader2);
    cfg.disconnect((leader2 + 1) % servers);
    thread::sleep(2 * RAFT_ELECTION_TIMEOUT);
    cfg.check_no_leader();

    // if a quorum arises, it should elect a leader.
    cfg.connect((leader2 + 1) % servers);
    cfg.check_one_leader();

    // re-join of last node shouldn't prevent leader from existing.
    cfg.connect(leader2);
    cfg.check_one_leader();

    cfg.end();
}

#[test]
fn test_basic_agree_2b() {
    let servers = 5;
    let mut cfg = Config::new(servers, false);
    cfg.begin("Test (2B): basic agreement");

    let iters = 3;
    for index in 1..=iters {
        let (nd, _) = cfg.n_committed(index);
        if nd > 0 {
            panic!("some have committed before start()");
        }

        let xindex = cfg.one(
            Entry {
                x: index as i64 * 100,
            },
            servers,
            false,
        );
        if xindex != index {
            panic!("got index {} but expected {}", xindex, index);
        }
    }

    cfg.end()
}

#[test]
fn test_fail_agree_2b() {
    let servers = 3;
    let mut cfg = Config::new(servers, false);

    cfg.begin("Test (2B): agreement despite follower disconnection");

    cfg.one(Entry { x: 101 }, servers, false);

    // follower network disconnection
    let leader = cfg.check_one_leader();
    cfg.disconnect((leader + 1) % servers);

    // agree despite one disconnected server?
    cfg.one(Entry { x: 102 }, servers - 1, false);
    cfg.one(Entry { x: 103 }, servers - 1, false);
    thread::sleep(RAFT_ELECTION_TIMEOUT);
    cfg.one(Entry { x: 104 }, servers - 1, false);
    cfg.one(Entry { x: 105 }, servers - 1, false);

    // re-connect
    cfg.connect((leader + 1) % servers);

    // agree with full set of servers?
    cfg.one(Entry { x: 106 }, servers, true);
    thread::sleep(RAFT_ELECTION_TIMEOUT);
    cfg.one(Entry { x: 107 }, servers, true);

    cfg.end();
}

#[test]
fn test_fail_no_agree_2b() {
    let servers = 5;
    let mut cfg = Config::new(servers, false);

    cfg.begin("Test (2B): no agreement if too many followers disconnect");

    cfg.one(Entry { x: 10 }, servers, false);

    // 3 of 5 followers disconnect
    let leader = cfg.check_one_leader();
    cfg.disconnect((leader + 1) % servers);
    cfg.disconnect((leader + 2) % servers);
    cfg.disconnect((leader + 3) % servers);
    let (index, _) = cfg.rafts[leader]
        .as_ref()
        .unwrap()
        .start(&20)
        .expect("leader rejected start");
    if index != 2 {
        panic!("expected index 2, got {}", index);
    }

    thread::sleep(2 * RAFT_ELECTION_TIMEOUT);

    let (n, _) = cfg.n_committed(index);
    if n > 0 {
        panic!("{} committed but no majority", n);
    }

    // repair
    cfg.connect((leader + 1) % servers);
    cfg.connect((leader + 2) % servers);
    cfg.connect((leader + 3) % servers);

    // the disconnected majority may have chosen a leader from
    // among their own ranks, forgetting index 2.
    let leader2 = cfg.check_one_leader();
    let (index2, _) = cfg.rafts[leader2]
        .as_ref()
        .unwrap()
        .start(&30)
        .expect("leader2 rejected start");
    if index2 < 2 || index2 > 3 {
        panic!("unexpected index {}", index2);
    }

    cfg.one(Entry { x: 1000 }, servers, true);

    cfg.end();
}

#[test]
fn test_concurrent_starts_2b() {
    let servers = 3;
    let mut cfg = Config::new(servers, false);

    cfg.begin("Test (2B): concurrent Start()s");
    let mut success = false;
    'outer: for try in 0..5 {
        if try > 0 {
            // give solution some time to settle
            thread::sleep(Duration::from_millis(3000));
        }

        let leader = cfg.check_one_leader();
        let term = match cfg.rafts[leader].as_ref().unwrap().start(&1) {
            Err(err) => {
                warn!("start leader {} meet error {:?}", leader, err);
                continue;
            }
            Ok((_, term)) => term,
        };

        let (tx, rx) = channel();
        let mut v = vec![];
        for i in 0..5 {
            let tx = tx.clone();
            let node = cfg.rafts[leader].as_ref().unwrap().clone();
            let child = thread::spawn(move || {
                match node.start(&(100 + i)) {
                    Err(err) => {
                        warn!("start leader {} meet error {:?}", leader, err);
                        return;
                    }
                    Ok((_, term1)) => {
                        if term1 != term {
                            return;
                        }
                    }
                };
                tx.send(i).unwrap();
            });
            v.push(child);
        }
        for child in v {
            child.join().unwrap();
        }
        drop(tx);

        for j in 0..servers {
            let t = cfg.rafts[j].as_ref().unwrap().term();
            if t != term {
                // term changed -- can't expect low RPC counts
                continue 'outer;
            }
        }

        let mut failed = false;
        let mut cmds = vec![];
        for index in rx.iter() {
            let cmd = cfg.wait(index, servers, Some(term)).unwrap();
            if cmd.x == -1 {
                // peers have moved on to later terms
                // so we can't expect all Start()s to
                // have succeeded
                failed = true;
                continue;
            }
            if !failed {
                cmds.push(cmd.x);
            }
        }

        for ii in 0..5 {
            let x = 100 + ii;
            let mut ok = false;
            for cmd in &cmds {
                if cmd == &x {
                    ok = true;
                }
            }
            if ok == false {
                panic!("cmd {} missing in {:?}", x, cmds)
            }
        }

        success = true;
        break;
    }

    if !success {
        panic!("term changed too often");
    }

    cfg.end();
}
