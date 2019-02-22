use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use futures::sync::oneshot;
use futures::{future, Future};
use futures_timer::Delay;
use rand::Rng;

use kvraft::client::Clerk;
use kvraft::config::Config;

/// The tester generously allows solutions to complete elections in one second
/// (much more than the paper's range of timeouts).
const RAFT_ELECTION_TIMEOUT: Duration = Duration::from_millis(1000);

// get/put/putappend that keep counts
fn get(cfg: &Config, ck: &Clerk, key: String) -> String {
    let v = ck.get(key);
    cfg.op();
    v
}

fn put(cfg: &Config, ck: &Clerk, key: String, value: String) {
    ck.put(key, value);
    cfg.op();
}

fn append(cfg: &Config, ck: &Clerk, key: String, value: String) {
    ck.append(key, value);
    cfg.op();
}

fn check(cfg: &Config, ck: &Clerk, key: String, value: String) {
    let v = get(cfg, ck, key.clone());
    if v != value {
        panic!("get({:?}): expected:\n{:?}\nreceived:\n{:?}", key, value, v);
    }
}

// spawn ncli clients and wait until they are all done
fn spawn_clients_and_wait<Func, Fact>(
    cfg: Arc<Config>,
    ncli: usize,
    fact: Fact,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    Fact: Fn() -> Func + Send + 'static,
    Func: Fn(usize, &Clerk) + Send + 'static,
{
    let mut cas = Vec::with_capacity(ncli);
    for cli in 0..ncli {
        let (tx, rx) = oneshot::channel();
        cas.push(rx.map(move |e| {
            debug!("spawn_clients_and_wait: client {} is done", cli);
        }));

        let cfg_ = cfg.clone();
        // a client runs the function func and then signals it is done
        let func = fact();
        cfg.net.spawn(future::lazy(move || {
            let ck = cfg_.make_client(&cfg_.all());
            func(cli, &ck);
            cfg_.delete_client(&ck);
            tx.send(())
        }));
    }
    debug!("spawn_clients_and_wait: waiting for clients");
    future::join_all(cas).map(|_| ()).map_err(|e| {
        panic!("spawn_clients_and_wait failed: {:?}", e);
    })
}

// predict effect of append(k, val) if old value is prev.
fn next_value(prev: String, val: &str) -> String {
    prev + val
}

// check that for a specific client all known appends are present in a value,
// and in order
fn check_clnt_appends(clnt: usize, v: String, count: usize) {
    let mut lastoff = None;
    for j in 0..count {
        let wanted = format!("x {} {} y", clnt, j);
        if let Some(off) = v.find(&wanted) {
            let off1 = v.rfind(&wanted).unwrap();
            assert_eq!(off1, off, "duplicate element {:?} in Append result", wanted);

            if let Some(lastoff) = lastoff {
                assert!( off > lastoff "wrong order for element {:?} in Append result", wanted);
            }
            lastoff = Some(off);
        } else {
            panic!(
                "{:?} missing element {:?} in Append result {:?}",
                clnt, wanted, v
            )
        }
    }
}

// check that all known appends are present in a value,
// and are in order for each concurrent client.
fn check_concurrent_appends(v: String, counts: &[usize]) {
    let nclients = counts.len();
    for i in 0..nclients {
        let mut lastoff = None;
        for (j, count) in counts.iter().enumerate() {
            let wanted = format!("x {} {} y", i, j);
            if let Some(off) = v.find(&wanted) {
                let off1 = v.rfind(&wanted).unwrap();
                assert_eq!(off1, off, "duplicate element {:?} in Append result", wanted);

                if let Some(lastoff) = lastoff {
                    assert!( off > lastoff "wrong order for element {:?} in Append result", wanted);
                }
                lastoff = Some(off);
            } else {
                panic!(
                    "{:?} missing element {:?} in Append result {:?}",
                    i, wanted, v
                )
            }
        }
    }
}

// repartition the servers periodically
fn partitioner(
    cfg: Arc<Config>,
    ch: mpsc::Sender<bool>,
    done: Arc<AtomicUsize>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    fn delay(r: u64) -> Delay {
        Delay::new(RAFT_ELECTION_TIMEOUT + Duration::from_millis(r % 200))
    }

    // Context of the poll_fn.
    let mut all = cfg.all();
    let mut sleep = None;
    let mut is_parked = false;
    future::poll_fn(move || {
        let mut rng = rand::thread_rng();
        while done.load(Ordering::Relaxed) == 0 {
            if !is_parked {
                rng.shuffle(&mut all);
                let offset = rng.gen_range(0, cfg.n);
                cfg.partition(&all[..offset], &all[offset..]);
                sleep = Some(delay(rng.gen::<u64>()));
            }
            is_parked = true;
            futures::try_ready!(sleep.as_mut().unwrap().poll().map_err(|e| {
                panic!("sleep failed: {:?}", e);
            }));
            is_parked = false;
        }
        ch.send(true).unwrap();
        Ok(futures::Async::Ready(()))
    })
}

// Basic test is as follows: one or more clients submitting Append/Get
// operations to set of servers for some period of time.  After the period is
// over, test checks that all appended values are present and in order for a
// particular key.  If unreliable is set, RPCs may fail.  If crash is set, the
// servers crash after the period is over and restart.  If partitions is set,
// the test repartitions the network concurrently with the clients and servers. If
// maxraftstate is a positive number, the size of the state for Raft (i.e., log
// size) shouldn't exceed 2*maxraftstate.
fn generic_test(
    part: &str,
    nclients: usize,
    unreliable: bool,
    crash: bool,
    partitions: bool,
    maxraftstate: Option<usize>,
) {
    let mut title = "Test: ".to_owned();
    if unreliable {
        // the network drops RPC requests and replies.
        title += "unreliable net, ";
    }
    if crash {
        // peers re-start, and thus persistence must work.
        title += "restarts, ";
    }
    if partitions {
        // the network may partition
        title += "partitions, ";
    }
    if maxraftstate.is_some() {
        title += "snapshots, ";
    }
    if nclients > 1 {
        title += "many clients";
    } else {
        title += "one client";
    }
    title = format!("{} ({})", title, part); // 3A or 3B

    const NSERVERS: usize = 5;
    let cfg = Arc::new(Config::new(NSERVERS, unreliable, maxraftstate));

    cfg.begin(&title);

    let ck = cfg.make_client(&cfg.all());

    let done_partitioner = Arc::new(AtomicUsize::new(0));
    let done_clients = Arc::new(AtomicUsize::new(0));
    let mut clnt_txs = vec![];
    let mut clnt_rxs = vec![];
    for i in 0..nclients {
        let (tx, rx) = mpsc::channel();
        clnt_txs.push(tx);
        clnt_rxs.push(rx);
    }
    for i in 0..3 {
        let (partitioner_tx, partitioner_rx) = mpsc::channel();
        debug!("Iteration {}", i);
        done_clients.store(0, Ordering::Relaxed);
        done_partitioner.store(0, Ordering::Relaxed);
        let clnt_txs_ = clnt_txs.clone();
        let cfg_ = cfg.clone();
        let done_clients_ = done_clients.clone();
        cfg.net
            .spawn_poller(spawn_clients_and_wait(cfg.clone(), nclients, move || {
                let cfg1 = cfg_.clone();
                let clnt_txs1 = clnt_txs_.clone();
                let done_clients1 = done_clients_.clone();
                move |cli, myck| {
                    // TODO: change the closure to a future.
                    let mut j = 0;
                    let mut rng = rand::thread_rng();
                    let mut last = String::new();
                    let key = format!("{}", cli);
                    put(&cfg1, myck, key.clone(), last.clone());
                    while done_clients1.load(Ordering::Relaxed) == 0 {
                        if (rng.gen::<u32>() % 1000) < 500 {
                            let nv = format!("x {} {} y", cli, i);
                            debug!("{}: client new append {}", cli, nv);
                            last = next_value(last, &nv);
                            append(&cfg1, myck, key.clone(), nv);
                            j += 1;
                        } else {
                            debug!("{}: client new get {:?}", cli, key);
                            let v = get(&cfg1, myck, key.clone());
                            if v != last {
                                panic!(
                                    "get wrong value, key {:?}, wanted:\n{:?}\n, got\n{:?}",
                                    key, last, v
                                );
                            }
                        }
                    }
                    clnt_txs1[cli].send(j).unwrap();
                }
            }));

        if partitions {
            // Allow the clients to perform some operations without interruption
            thread::sleep(Duration::from_secs(1));
            cfg.net.spawn_poller(partitioner(
                cfg.clone(),
                partitioner_tx,
                done_partitioner.clone(),
            ));
        }
        thread::sleep(Duration::from_secs(5));

        // tell clients to quit
        done_clients.store(1, Ordering::Relaxed);
        // tell partitioner to quit
        done_partitioner.store(1, Ordering::Relaxed);

        if partitions {
            debug!("wait for partitioner");
            partitioner_rx.try_recv().unwrap();
            // reconnect network and submit a request. A client may
            // have submitted a request in a minority.  That request
            // won't return until that server discovers a new term
            // has started.
            cfg.connect_all();
            // wait for a while so that we have a new term
            thread::sleep(RAFT_ELECTION_TIMEOUT);
        }

        if crash {
            debug!("shutdown servers");
            for i in 0..NSERVERS {
                cfg.shutdown_server(i)
            }
            // Wait for a while for servers to shutdown, since
            // shutdown isn't a real crash and isn't instantaneous
            thread::sleep(RAFT_ELECTION_TIMEOUT);
            debug!("restart servers");
            // crash and re-start all
            for i in 0..NSERVERS {
                cfg.start_server(i);
            }
            cfg.connect_all();
        }

        debug!("wait for clients");
        for (i, clnt_rx) in clnt_rxs.iter().enumerate() {
            debug!("read from clients {}", i);
            let j = clnt_rx.try_recv().unwrap();
            if j < 10 {
                debug!(
                    "Warning: client {} managed to perform only {} put operations in 1 sec?",
                    i, j
                );
            }
            let key = format!("{}", i);
            debug!("Check {:?} for client {}", j, i);
            let v = get(&cfg, &ck, key);
            check_clnt_appends(i, v, j);
        }

        if let Some(maxraftstate) = maxraftstate {
            // Check maximum after the servers have processed all client
            // requests and had time to checkpoint.
            if cfg.log_size() > 2 * maxraftstate {
                panic!(
                    "logs were not trimmed ({} > 2*{})",
                    cfg.log_size(),
                    maxraftstate
                )
            }
        }
    }

    cfg.check_timeout();
    cfg.end();
}

#[test]
fn test_basic_3a() {
    // Test: one client (3A) ...
    generic_test("3A", 1, false, false, false, None)
}

#[test]
fn test_concurrent_3a() {
    // Test: many clients (3A) ...
    generic_test("3A", 5, false, false, false, None)
}

#[test]
fn test_unreliable_3a() {
    // Test: unreliable net, many clients (3A) ...
    generic_test("3A", 5, true, false, false, None)
}

#[test]
fn test_many_partitions_one_client_3a() {
    // Test: partitions, one client (3A) ...
    generic_test("3A", 1, false, false, true, None)
}

#[test]
fn test_many_partitions_many_clients_3a() {
    // Test: partitions, many clients (3A) ...
    generic_test("3A", 5, false, false, true, None)
}

#[test]
fn test_persist_one_client_3a() {
    // Test: restarts, one client (3A) ...
    generic_test("3A", 1, false, true, false, None)
}

#[test]
fn test_persist_concurrent_3a() {
    // Test: restarts, many clients (3A) ...
    generic_test("3A", 5, false, true, false, None)
}

#[test]
fn test_persist_concurrent_unreliable_3a() {
    // Test: unreliable net, restarts, many clients (3A) ...
    generic_test("3A", 5, true, true, false, None)
}

#[test]
fn test_persist_partition_3a() {
    // Test: restarts, partitions, many clients (3A) ...
    generic_test("3A", 5, false, true, true, None)
}

#[test]
fn test_persist_partition_unreliable_3a() {
    // Test: unreliable net, restarts, partitions, many clients (3A) ...
    generic_test("3A", 5, true, true, true, None)
}

#[test]
fn test_snapshot_recover_3b() {
    // Test: restarts, snapshots, one client (3B) ...
    generic_test("3B", 1, false, true, false, Some(1000))
}

#[test]
fn test_snapshot_recover_many_clients_3b() {
    // Test: restarts, snapshots, many clients (3B) ...
    generic_test("3B", 20, false, true, false, Some(1000))
}

#[test]
fn test_snapshot_unreliable_3b() {
    // Test: unreliable net, snapshots, many clients (3B) ...
    generic_test("3B", 5, true, false, false, Some(1000))
}

#[test]
fn test_snapshot_unreliable_recover_3b() {
    // Test: unreliable net, restarts, snapshots, many clients (3B) ...
    generic_test("3B", 5, true, true, false, Some(1000))
}

#[test]
fn test_snapshot_unreliable_recover_concurrent_partition_3b() {
    // Test: unreliable net, restarts, partitions, snapshots, many clients (3B) ...
    generic_test("3B", 5, true, true, true, Some(1000))
}
