#![allow(clippy::identity_op)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use futures::channel::oneshot;
use futures::executor::block_on;
use futures::future;
use rand::{rngs::ThreadRng, Rng};

use crate::raft::config::{Config, Entry, Storage, SNAPSHOT_INTERVAL};
use crate::raft::Node;

/// The tester generously allows solutions to complete elections in one second
/// (much more than the paper's range of timeouts).
const RAFT_ELECTION_TIMEOUT: Duration = Duration::from_millis(1000);

fn random_entry(rnd: &mut ThreadRng) -> Entry {
    Entry {
        x: rnd.gen::<u64>(),
    }
}

#[test]
fn test_initial_election_2a() {
    let servers = 3;
    let mut cfg = Config::new(servers);

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
    let mut cfg = Config::new(servers);
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
fn test_many_election_2a() {
    let servers = 7;
    let iters = 10;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2A): multiple elections");

    cfg.check_one_leader();

    let mut random = rand::thread_rng();
    for _ in 0..iters {
        // disconnect three nodes
        let i1 = random.gen::<usize>() % servers;
        let i2 = random.gen::<usize>() % servers;
        let i3 = random.gen::<usize>() % servers;
        cfg.disconnect(i1);
        cfg.disconnect(i2);
        cfg.disconnect(i3);

        // either the current leader should still be alive,
        // or the remaining four should elect a new one.
        cfg.check_one_leader();

        cfg.connect(i1);
        cfg.connect(i2);
        cfg.connect(i3);
    }

    cfg.check_one_leader();

    cfg.end();
}

#[test]
fn test_basic_agree_2b() {
    let servers = 5;
    let mut cfg = Config::new(servers);
    cfg.begin("Test (2B): basic agreement");

    let iters = 3;
    for index in 1..=iters {
        let (nd, _) = cfg.n_committed(index);
        if nd > 0 {
            panic!("some have committed before start()");
        }

        let xindex = cfg.one(Entry { x: index * 100 }, servers, false);
        if xindex != index {
            panic!("got index {} but expected {}", xindex, index);
        }
    }

    cfg.end()
}

#[test]
fn test_fail_agree_2b() {
    let servers = 3;
    let mut cfg = Config::new(servers);

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
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2B): no agreement if too many followers disconnect");

    cfg.one(Entry { x: 10 }, servers, false);

    // 3 of 5 followers disconnect
    let leader = cfg.check_one_leader();
    cfg.disconnect((leader + 1) % servers);
    cfg.disconnect((leader + 2) % servers);
    cfg.disconnect((leader + 3) % servers);
    let (index, _) = cfg.rafts.lock().unwrap()[leader]
        .as_ref()
        .unwrap()
        .start(&Entry { x: 20 })
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
    let (index2, _) = cfg.rafts.lock().unwrap()[leader2]
        .as_ref()
        .unwrap()
        .start(&Entry { x: 30 })
        .expect("leader2 rejected start");
    if !(2..=3).contains(&index2) {
        panic!("unexpected index {}", index2);
    }

    cfg.one(Entry { x: 1000 }, servers, true);

    cfg.end();
}

#[test]
fn test_concurrent_starts_2b() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2B): concurrent start()s");
    let mut success = false;
    'outer: for tried in 0..5 {
        if tried > 0 {
            // give solution some time to settle
            thread::sleep(Duration::from_secs(3));
        }

        let leader = cfg.check_one_leader();
        let term = match cfg.rafts.lock().unwrap()[leader]
            .as_ref()
            .unwrap()
            .start(&Entry { x: 1 })
        {
            Err(err) => {
                warn!("start leader {} meet error {:?}", leader, err);
                continue;
            }
            Ok((_, term)) => term,
        };

        let mut idx_rxs = vec![];
        for ii in 0..5 {
            let (tx, rx) = oneshot::channel();
            idx_rxs.push(rx);
            let node = cfg.rafts.lock().unwrap()[leader].clone().unwrap();
            cfg.net.spawn(future::lazy(move |_| {
                let idx = match node.start(&Entry { x: 100 + ii }) {
                    Err(err) => {
                        warn!("start leader {} meet error {:?}", leader, err);
                        None
                    }
                    Ok((idx, term1)) => {
                        if term1 != term {
                            None
                        } else {
                            Some(idx)
                        }
                    }
                };
                tx.send(idx)
                    .map_err(|e| panic!("send failed: {:?}", e))
                    .unwrap();
            }));
        }
        let idxes = block_on(async {
            future::join_all(idx_rxs)
                .await
                .into_iter()
                .map(|idx_rx| idx_rx.unwrap())
                .collect::<Vec<_>>()
        });

        for j in 0..servers {
            let t = cfg.rafts.lock().unwrap()[j].as_ref().unwrap().term();
            if t != term {
                // term changed -- can't expect low RPC counts
                continue 'outer;
            }
        }

        let mut cmds = vec![];
        for index in idxes.into_iter().flatten() {
            if let Some(cmd) = cfg.wait(index, servers, Some(term)) {
                cmds.push(cmd.x);
            } else {
                // peers have moved on to later terms
                // so we can't expect all Start()s to
                // have succeeded
                continue;
            }
        }

        for ii in 0..5 {
            let x = 100 + ii;
            let mut ok = false;
            for cmd in &cmds {
                if *cmd == x {
                    ok = true;
                }
            }
            assert!(ok, "cmd {} missing in {:?}", x, cmds)
        }

        success = true;
        break;
    }

    assert!(success, "term changed too often");

    cfg.end();
}

#[test]
fn test_rejoin_2b() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2B): rejoin of partitioned leader");

    cfg.one(Entry { x: 101 }, servers, true);

    // leader network failure
    let leader1 = cfg.check_one_leader();
    cfg.disconnect(leader1);

    // make old leader try to agree on some entries
    let _ = cfg.rafts.lock().unwrap()[leader1]
        .as_ref()
        .unwrap()
        .start(&Entry { x: 102 });
    let _ = cfg.rafts.lock().unwrap()[leader1]
        .as_ref()
        .unwrap()
        .start(&Entry { x: 103 });
    let _ = cfg.rafts.lock().unwrap()[leader1]
        .as_ref()
        .unwrap()
        .start(&Entry { x: 104 });

    // new leader commits, also for index=2
    cfg.one(Entry { x: 103 }, 2, true);

    // new leader network failure
    let leader2 = cfg.check_one_leader();
    cfg.disconnect(leader2);

    // old leader connected again
    cfg.connect(leader1);

    cfg.one(Entry { x: 104 }, 2, true);

    // all together now
    cfg.connect(leader2);

    cfg.one(Entry { x: 105 }, servers, true);

    cfg.end();
}

#[test]
fn test_backup_2b() {
    let servers = 5;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2B): leader backs up quickly over incorrect follower logs");

    let mut random = rand::thread_rng();
    cfg.one(random_entry(&mut random), servers, true);

    // put leader and one follower in a partition
    let leader1 = cfg.check_one_leader();
    cfg.disconnect((leader1 + 2) % servers);
    cfg.disconnect((leader1 + 3) % servers);
    cfg.disconnect((leader1 + 4) % servers);

    // submit lots of commands that won't commit
    for _i in 0..50 {
        let _ = cfg.rafts.lock().unwrap()[leader1]
            .as_ref()
            .unwrap()
            .start(&random_entry(&mut random));
    }

    thread::sleep(RAFT_ELECTION_TIMEOUT / 2);

    cfg.disconnect((leader1 + 0) % servers);
    cfg.disconnect((leader1 + 1) % servers);

    // allow other partition to recover
    cfg.connect((leader1 + 2) % servers);
    cfg.connect((leader1 + 3) % servers);
    cfg.connect((leader1 + 4) % servers);

    // lots of successful commands to new group.
    for _i in 0..50 {
        cfg.one(random_entry(&mut random), 3, true);
    }

    // now another partitioned leader and one follower
    let leader2 = cfg.check_one_leader();
    let mut other = (leader1 + 2) % servers;
    if leader2 == other {
        other = (leader2 + 1) % servers;
    }
    cfg.disconnect(other);

    // lots more commands that won't commit
    for _i in 0..50 {
        let _ = cfg.rafts.lock().unwrap()[leader2]
            .as_ref()
            .unwrap()
            .start(&random_entry(&mut random));
    }

    thread::sleep(RAFT_ELECTION_TIMEOUT / 2);

    // bring original leader back to life,
    for i in 0..servers {
        cfg.disconnect(i);
    }
    cfg.connect((leader1 + 0) % servers);
    cfg.connect((leader1 + 1) % servers);
    cfg.connect(other);

    // lots of successful commands to new group.
    for _i in 0..50 {
        cfg.one(random_entry(&mut random), 3, true);
    }

    // now everyone
    for i in 0..servers {
        cfg.connect(i);
    }
    cfg.one(random_entry(&mut random), servers, true);

    cfg.end();
}

#[test]
fn test_count_2b() {
    const SERVERS: usize = 3;
    fn rpcs(cfg: &Config) -> usize {
        let mut n: usize = 0;
        for j in 0..SERVERS {
            n += cfg.rpc_count(j);
        }
        n
    }

    let mut cfg = Config::new(SERVERS);

    cfg.begin("Test (2B): RPC counts aren't too high");

    cfg.check_one_leader();
    let mut total1 = rpcs(&cfg);

    if !(1..=30).contains(&total1) {
        panic!("too many or few RPCs ({}) to elect initial leader", total1);
    }

    let mut total2 = 0;
    let mut success = false;
    'outer: for tried in 0..5 {
        if tried > 0 {
            // give solution some time to settle
            thread::sleep(Duration::from_secs(3));
        }

        let leader = cfg.check_one_leader();
        total1 = rpcs(&cfg);

        let iters = 10;
        let (starti, term) = match cfg.rafts.lock().unwrap()[leader]
            .as_ref()
            .unwrap()
            .start(&Entry { x: 1 })
        {
            Ok((starti, term)) => (starti, term),
            Err(err) => {
                warn!("start leader {} meet error {:?}", leader, err);
                continue;
            }
        };

        let mut cmds = vec![];
        let mut random = rand::thread_rng();
        for i in 1..iters + 2 {
            let x = random.gen::<u64>();
            cmds.push(x);
            match cfg.rafts.lock().unwrap()[leader]
                .as_ref()
                .unwrap()
                .start(&Entry { x })
            {
                Ok((index1, term1)) => {
                    if term1 != term {
                        // Term changed while starting
                        continue 'outer;
                    }
                    if starti + i != index1 {
                        panic!("start failed");
                    }
                }
                Err(err) => {
                    warn!("start leader {} meet error {:?}", leader, err);
                    continue 'outer;
                }
            }
        }

        for i in 1..=iters {
            if let Some(ix) = cfg.wait(starti + i, SERVERS, Some(term)) {
                if ix.x != cmds[(i - 1) as usize] {
                    panic!(
                        "wrong value {:?} committed for index {}; expected {:?}",
                        ix,
                        starti + i,
                        cmds
                    );
                }
            }
        }

        let mut failed = false;
        total2 = 0;
        for j in 0..SERVERS {
            let t = cfg.rafts.lock().unwrap()[j].as_ref().unwrap().term();
            if t != term {
                // term changed -- can't expect low RPC counts
                // need to keep going to update total2
                failed = true;
            }
            total2 += cfg.rpc_count(j);
        }

        if failed {
            continue 'outer;
        }

        if total2 - total1 > (iters as usize + 1 + 3) * 3 {
            panic!("too many RPCs ({}) for {} entries", total2 - total1, iters);
        }

        success = true;
        break;
    }

    if !success {
        panic!("term changed too often");
    }

    thread::sleep(RAFT_ELECTION_TIMEOUT);

    let mut total3 = 0;
    for j in 0..SERVERS {
        total3 += cfg.rpc_count(j);
    }

    if total3 - total2 > 3 * 20 {
        panic!(
            "too many RPCs ({}) for 1 second of idleness",
            total3 - total2
        );
    }
    cfg.end();
}

#[test]
fn test_persist1_2c() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2C): basic persistence");

    cfg.one(Entry { x: 11 }, servers, true);

    // crash and re-start all
    for i in 0..servers {
        cfg.start1(i);
    }
    for i in 0..servers {
        cfg.disconnect(i);
        cfg.connect(i);
    }

    cfg.one(Entry { x: 12 }, servers, true);

    let leader1 = cfg.check_one_leader();
    cfg.disconnect(leader1);
    cfg.start1(leader1);
    cfg.connect(leader1);

    cfg.one(Entry { x: 13 }, servers, true);

    let leader2 = cfg.check_one_leader();
    cfg.disconnect(leader2);
    cfg.one(Entry { x: 14 }, servers - 1, true);
    cfg.start1(leader2);
    cfg.connect(leader2);

    cfg.wait(4, servers, None); // wait for leader2 to join before killing i3

    let i3 = (cfg.check_one_leader() + 1) % servers;
    cfg.disconnect(i3);
    cfg.one(Entry { x: 15 }, servers - 1, true);
    cfg.start1(i3);
    cfg.connect(i3);

    cfg.one(Entry { x: 16 }, servers, true);

    cfg.end();
}

#[test]
fn test_persist2_2c() {
    let servers = 5;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2C): more persistence");

    let mut index = 1;
    for _ in 0..5 {
        cfg.one(Entry { x: 10 + index }, servers, true);
        index += 1;

        let leader1 = cfg.check_one_leader();

        cfg.disconnect((leader1 + 1) % servers);
        cfg.disconnect((leader1 + 2) % servers);

        cfg.one(Entry { x: 10 + index }, servers - 2, true);
        index += 1;

        cfg.disconnect((leader1 + 0) % servers);
        cfg.disconnect((leader1 + 3) % servers);
        cfg.disconnect((leader1 + 4) % servers);

        cfg.start1((leader1 + 1) % servers);
        cfg.start1((leader1 + 2) % servers);
        cfg.connect((leader1 + 1) % servers);
        cfg.connect((leader1 + 2) % servers);

        thread::sleep(RAFT_ELECTION_TIMEOUT);

        cfg.start1((leader1 + 3) % servers);
        cfg.connect((leader1 + 3) % servers);

        cfg.one(Entry { x: 10 + index }, servers - 2, true);
        index += 1;

        cfg.connect((leader1 + 4) % servers);
        cfg.connect((leader1 + 0) % servers);
    }

    cfg.one(Entry { x: 1000 }, servers, true);

    cfg.end();
}

#[test]
fn test_persist3_2c() {
    let servers = 3;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2C): partitioned leader and one follower crash, leader restarts");

    cfg.one(Entry { x: 101 }, 3, true);

    let leader = cfg.check_one_leader();
    cfg.disconnect((leader + 2) % servers);

    cfg.one(Entry { x: 102 }, 2, true);

    cfg.crash1((leader + 0) % servers);
    cfg.crash1((leader + 1) % servers);
    cfg.connect((leader + 2) % servers);
    cfg.start1((leader + 0) % servers);
    cfg.connect((leader + 0) % servers);

    cfg.one(Entry { x: 103 }, 2, true);

    cfg.start1((leader + 1) % servers);
    cfg.connect((leader + 1) % servers);

    cfg.one(Entry { x: 104 }, servers, true);

    cfg.end();
}

// Test the scenarios described in Figure 8 of the extended Raft paper. Each
// iteration asks a leader, if there is one, to insert a command in the Raft
// log.  If there is a leader, that leader will fail quickly with a high
// probability (perhaps without committing the command), or crash after a while
// with low probability (most likey committing the command).  If the number of
// alive servers isn't enough to form a majority, perhaps start a new server.
// The leader in a new term may try to finish replicating log entries that
// haven't been committed yet.
#[test]
fn test_figure_8_2c() {
    let servers = 5;
    let mut cfg = Config::new(servers);

    cfg.begin("Test (2C): Figure 8");

    let mut random = rand::thread_rng();
    cfg.one(random_entry(&mut random), 1, true);

    let mut nup = servers;
    for _iters in 0..1000 {
        let mut leader = None;
        for i in 0..servers {
            let mut rafts = cfg.rafts.lock().unwrap();
            if let Some(Some(raft)) = rafts.get_mut(i) {
                if raft.start(&random_entry(&mut random)).is_ok() {
                    leader = Some(i);
                }
            }
        }

        if (random.gen::<usize>() % 1000) < 100 {
            let ms = random.gen::<u64>() % ((RAFT_ELECTION_TIMEOUT.as_millis() / 2) as u64);
            thread::sleep(Duration::from_millis(ms));
        } else {
            let ms = random.gen::<u64>() % 13;
            thread::sleep(Duration::from_millis(ms));
        }

        if let Some(leader) = leader {
            cfg.crash1(leader);
            nup -= 1;
        }

        if nup < 3 {
            let s = random.gen::<usize>() % servers;
            if cfg.rafts.lock().unwrap().get(s).unwrap().is_none() {
                cfg.start1(s);
                cfg.connect(s);
                nup += 1;
            }
        }
    }

    for i in 0..servers {
        if cfg.rafts.lock().unwrap().get(i).unwrap().is_none() {
            cfg.start1(i);
            cfg.connect(i);
        }
    }

    cfg.one(random_entry(&mut random), servers, true);

    cfg.end();
}

#[test]
fn test_unreliable_agree_2c() {
    let servers = 5;

    let cfg = {
        let mut cfg = Config::new_with(servers, true, false);
        cfg.begin("Test (2C): unreliable agreement");
        Arc::new(cfg)
    };

    let mut dones = vec![];
    for iters in 1..50 {
        for j in 0..4 {
            let c = cfg.clone();
            let (tx, rx) = oneshot::channel();
            thread::spawn(move || {
                c.one(
                    Entry {
                        x: (100 * iters) + j,
                    },
                    1,
                    true,
                );
                tx.send(()).map_err(|e| panic!("send failed: {:?}", e))
            });
            dones.push(rx);
        }
        cfg.one(Entry { x: iters }, 1, true);
    }

    cfg.net.set_reliable(true);

    block_on(async {
        future::join_all(dones)
            .await
            .into_iter()
            .for_each(|done| done.unwrap());
    });

    cfg.one(Entry { x: 100 }, servers, true);

    cfg.end();
}

#[test]
fn test_figure_8_unreliable_2c() {
    let servers = 5;
    let mut cfg = Config::new_with(servers, true, false);

    cfg.begin("Test (2C): Figure 8 (unreliable)");
    let mut random = rand::thread_rng();
    cfg.one(
        Entry {
            x: random.gen::<u64>() % 10000,
        },
        1,
        true,
    );

    let mut nup = servers;
    for iters in 0..1000 {
        if iters == 200 {
            cfg.net.set_long_reordering(true);
        }
        let mut leader = None;
        for i in 0..servers {
            if cfg.rafts.lock().unwrap()[i]
                .as_ref()
                .unwrap()
                .start(&Entry {
                    x: random.gen::<u64>() % 10000,
                })
                .is_ok()
                && cfg.connected[i]
            {
                leader = Some(i);
            }
        }

        if (random.gen::<usize>() % 1000) < 100 {
            let ms = random.gen::<u64>() % (RAFT_ELECTION_TIMEOUT.as_millis() as u64 / 2);
            thread::sleep(Duration::from_millis(ms as u64));
        } else {
            let ms = random.gen::<u64>() % 13;
            thread::sleep(Duration::from_millis(ms));
        }

        if let Some(leader) = leader {
            if (random.gen::<usize>() % 1000) < (RAFT_ELECTION_TIMEOUT.as_millis() as usize) / 2 {
                cfg.disconnect(leader);
                nup -= 1;
            }
        }

        if nup < 3 {
            let s = random.gen::<usize>() % servers;
            if !cfg.connected[s] {
                cfg.connect(s);
                nup += 1;
            }
        }
    }

    for i in 0..servers {
        if !cfg.connected[i] {
            cfg.connect(i);
        }
    }

    cfg.one(
        Entry {
            x: random.gen::<u64>() % 10000,
        },
        servers,
        true,
    );

    cfg.end();
}

fn internal_churn(unreliable: bool) {
    let servers = 5;
    let mut cfg = Config::new_with(servers, unreliable, false);
    if unreliable {
        cfg.begin("Test (2C): unreliable churn")
    } else {
        cfg.begin("Test (2C): churn")
    }

    let stop = Arc::new(AtomicUsize::new(0));

    // create concurrent clients
    // TODO: change it a future
    fn cfn(
        me: usize,
        stop_clone: Arc<AtomicUsize>,
        tx: Sender<Option<Vec<u64>>>,
        rafts: Arc<Mutex<Box<[Option<Node>]>>>,
        storage: Arc<Mutex<Storage>>,
    ) {
        let mut values = vec![];
        while stop_clone.load(Ordering::SeqCst) == 0 {
            let mut random = rand::thread_rng();
            let x = random.gen::<u64>();
            let mut index: i64 = -1;
            let mut ok = false;
            // try them all, maybe one of them is a leader
            let rafts: Vec<_> = rafts.lock().unwrap().iter().cloned().collect();
            for raft in &rafts {
                match raft {
                    Some(rf) => {
                        match rf.start(&Entry { x }) {
                            Ok((index1, _)) => {
                                index = index1 as i64;
                                ok = true;
                            }
                            Err(_) => continue,
                        };
                    }
                    None => continue,
                }
            }
            if ok {
                // maybe leader will commit our value, maybe not.
                // but don't wait forever.
                for to in &[10, 20, 50, 100, 200] {
                    let (nd, cmd) = storage.lock().unwrap().n_committed(index as u64);
                    if nd > 0 {
                        match cmd {
                            Some(xx) => {
                                if xx.x == x {
                                    values.push(xx.x);
                                }
                            }
                            None => panic!("wrong command type"),
                        }
                        break;
                    }
                    thread::sleep(Duration::from_millis(*to));
                }
            } else {
                thread::sleep(Duration::from_millis((79 + me * 17) as u64));
            }
        }
        if !values.is_empty() {
            tx.send(Some(values)).unwrap();
        } else {
            tx.send(None).unwrap();
        }
    }

    let ncli = 3;
    let mut nrec = vec![];
    for i in 0..ncli {
        let stop_clone = stop.clone();
        let (tx, rx) = channel();
        let storage = cfg.storage.clone();
        let rafts = cfg.rafts.clone();
        thread::spawn(move || {
            cfn(i, stop_clone, tx, rafts, storage);
        });
        nrec.push(rx);
    }
    let mut random = rand::thread_rng();
    for _iters in 0..20 {
        if (random.gen::<usize>() % 1000) < 200 {
            let i = random.gen::<usize>() % servers;
            cfg.disconnect(i);
        }

        if (random.gen::<usize>() % 1000) < 500 {
            let i = random.gen::<usize>() % servers;
            if cfg.rafts.lock().unwrap().get(i).unwrap().is_none() {
                cfg.start1(i);
            }
            cfg.connect(i);
        }

        if (random.gen::<usize>() % 1000) < 200 {
            let i = random.gen::<usize>() % servers;
            if cfg.rafts.lock().unwrap().get(i).unwrap().is_some() {
                cfg.crash1(i);
            }
        }

        // Make crash/restart infrequent enough that the peers can often
        // keep up, but not so infrequent that everything has settled
        // down from one change to the next. Pick a value smaller than
        // the election timeout, but not hugely smaller.
        thread::sleep((RAFT_ELECTION_TIMEOUT * 7) / 10)
    }

    thread::sleep(RAFT_ELECTION_TIMEOUT);
    cfg.net.set_reliable(true);
    for i in 0..servers {
        if cfg.rafts.lock().unwrap().get(i).unwrap().is_none() {
            cfg.start1(i);
        }
        cfg.connect(i);
    }

    stop.store(1, Ordering::SeqCst);

    let mut values = vec![];
    for rx in &nrec {
        let mut vv = rx.recv().unwrap().unwrap();
        values.append(&mut vv);
    }

    thread::sleep(RAFT_ELECTION_TIMEOUT);

    let last_index = cfg.one(random_entry(&mut random), servers, true);

    let mut really = vec![];
    for index in 1..=last_index {
        let v = cfg.wait(index, servers, None).unwrap();
        really.push(v.x);
    }

    for v1 in &values {
        let mut ok = false;
        for v2 in &really {
            if v1 == v2 {
                ok = true;
            }
        }
        assert!(ok, "didn't find a value");
    }

    cfg.end()
}

#[test]
fn test_reliable_churn_2c() {
    internal_churn(false);
}

#[test]
fn test_unreliable_churn_2c() {
    internal_churn(true);
}

fn snap_common(name: &str, disconnect: bool, reliable: bool, crash: bool) {
    const MAX_LOG_SIZE: usize = 2000;

    let iters = 30;
    let servers = 3;
    let mut cfg = Config::new_with(servers, !reliable, true);

    cfg.begin(name);

    let mut random = rand::thread_rng();
    cfg.one(random_entry(&mut random), servers, true);
    let mut leader1 = cfg.check_one_leader();

    for i in 0..iters {
        let mut victim = (leader1 + 1) % servers;
        let mut sender = leader1;
        if i % 3 == 1 {
            sender = (leader1 + 1) % servers;
            victim = leader1;
        }

        if disconnect {
            cfg.disconnect(victim);
            cfg.one(random_entry(&mut random), servers - 1, true);
        }
        if crash {
            cfg.crash1(victim);
            cfg.one(random_entry(&mut random), servers - 1, true);
        }
        // send enough to get a snapshot
        for _ in 0..=SNAPSHOT_INTERVAL {
            let _ = cfg.rafts.lock().unwrap()[sender]
                .as_ref()
                .unwrap()
                .start(&random_entry(&mut random));
        }
        // let applier threads catch up with the Start()'s
        cfg.one(random_entry(&mut random), servers - 1, true);

        assert!(cfg.log_size() < MAX_LOG_SIZE, "log size too large");

        if disconnect {
            // reconnect a follower, who maybe behind and
            // needs to receive a snapshot to catch up.
            cfg.connect(victim);
            cfg.one(random_entry(&mut random), servers, true);
            leader1 = cfg.check_one_leader();
        }
        if crash {
            cfg.start1_snapshot(victim);
            cfg.connect(victim);
            cfg.one(random_entry(&mut random), servers, true);
            leader1 = cfg.check_one_leader();
        }
    }
    cfg.end();
}

#[test]
fn test_snapshot_basic_2d() {
    snap_common("Test (2D): snapshots basic", false, true, false);
}

#[test]
fn test_snapshot_install_2d() {
    snap_common(
        "Test (2D): install snapshots (disconnect)",
        true,
        true,
        false,
    );
}

#[test]
fn test_snapshot_install_unreliable_2d() {
    snap_common(
        "Test (2D): install snapshots (disconnect+unreliable)",
        true,
        false,
        false,
    );
}

#[test]
fn test_snapshot_install_crash_2d() {
    snap_common("Test (2D): install snapshots (crash)", false, true, true);
}

#[test]
fn test_snapshot_install_unreliable_crash_2d() {
    snap_common(
        "Test (2D): install snapshots (unreliable+crash)",
        false,
        false,
        true,
    );
}
