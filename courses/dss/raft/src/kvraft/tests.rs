use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Poll;
use std::thread;
use std::time::{Duration, Instant};

use futures::channel::oneshot;
use futures::executor::block_on;
use futures::future;
use futures::{Future, FutureExt};
use futures_timer::Delay;
use rand::{seq::SliceRandom, Rng};

use linearizability::check_operations_timeout;
use linearizability::model::Operation;
use linearizability::models::{KvInput, KvModel, KvOutput, Op};

use crate::kvraft::client::Clerk;
use crate::kvraft::config::Config;

/// The tester generously allows solutions to complete elections in one second
/// (much more than the paper's range of timeouts).
const RAFT_ELECTION_TIMEOUT: Duration = Duration::from_millis(1000);

const LINEARIZABILITY_CHECK_TIMEOUT: Duration = Duration::from_millis(1000);

// get/put/append that keep counts
fn get(cfg: &Config, ck: &Clerk, key: &str) -> String {
    let v = ck.get(key.to_owned());
    cfg.op();
    v
}

fn put(cfg: &Config, ck: &Clerk, key: &str, value: &str) {
    ck.put(key.to_owned(), value.to_owned());
    cfg.op();
}

fn append(cfg: &Config, ck: &Clerk, key: &str, value: &str) {
    ck.append(key.to_owned(), value.to_owned());
    cfg.op();
}

fn check(cfg: &Config, ck: &Clerk, key: &str, value: &str) {
    let v = get(cfg, ck, key);
    if v != value {
        panic!("get({:?}): expected:\n{:?}\nreceived:\n{:?}", key, value, v);
    }
}

// spawn ncli clients and wait until they are all done
fn spawn_clients_and_wait<Func, Fact>(
    cfg: Arc<Config>,
    ncli: usize,
    fact: Fact,
) -> impl Future<Output = ()> + Send + 'static
where
    Fact: Fn() -> Func + Send + 'static,
    Func: Fn(usize, &Clerk) + Send + 'static,
{
    let mut cas = Vec::with_capacity(ncli);
    for cli in 0..ncli {
        let (tx, rx) = oneshot::channel();
        cas.push(rx.map(move |_| {
            debug!("spawn_clients_and_wait: client {} is done", cli);
        }));

        let cfg_ = cfg.clone();
        // a client runs the function func and then signals it is done
        let func = fact();
        thread::spawn(move || {
            let ck = cfg_.make_client(&cfg_.all());
            func(cli, &ck);
            cfg_.delete_client(&ck);
            tx.send(())
        });
    }
    debug!("spawn_clients_and_wait: waiting for clients");
    future::join_all(cas).map(|_| ())
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
                assert!(
                    off > lastoff,
                    "wrong order for element {:?} in Append result",
                    wanted
                );
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
#[allow(clippy::needless_range_loop)]
fn check_concurrent_appends(v: String, counts: &[usize]) {
    let nclients = counts.len();
    for i in 0..nclients {
        let mut lastoff = None;
        for j in 0..counts[i] {
            let wanted = format!("x {} {} y", i, j);
            if let Some(off) = v.find(&wanted) {
                let off1 = v.rfind(&wanted).unwrap();
                assert_eq!(off1, off, "duplicate element {:?} in Append result", wanted);

                if let Some(lastoff) = lastoff {
                    assert!(
                        off > lastoff,
                        "wrong order for element {:?} in Append result",
                        wanted
                    );
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
) -> impl Future<Output = ()> + Send + 'static {
    fn delay(r: u64) -> Delay {
        Delay::new(RAFT_ELECTION_TIMEOUT + Duration::from_millis(r % 200))
    }

    // Context of the poll_fn.
    let mut all = cfg.all();
    let mut sleep = None;
    let mut is_parked = false;
    future::poll_fn(move |cx| {
        let mut rng = rand::thread_rng();
        while done.load(Ordering::Relaxed) == 0 {
            if !is_parked {
                all.shuffle(&mut rng);
                let offset = rng.gen_range(0, cfg.n);
                cfg.partition(&all[..offset], &all[offset..]);
                sleep = Some(delay(rng.gen::<u64>()));
            }
            is_parked = true;
            let sleep = sleep.as_mut().unwrap();
            futures::pin_mut!(sleep);
            futures::ready!(sleep.poll(cx));
            is_parked = false;
        }
        ch.send(true).unwrap();
        Poll::Ready(())
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
    for _ in 0..nclients {
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
        thread::spawn(move || {
            block_on(async {
                spawn_clients_and_wait(cfg_.clone(), nclients, move || {
                    let cfg1 = cfg_.clone();
                    let clnt_txs1 = clnt_txs_.clone();
                    let done_clients1 = done_clients_.clone();
                    move |cli, myck| {
                        // TODO: change the closure to a future.
                        let mut j = 0;
                        let mut rng = rand::thread_rng();
                        let mut last = String::new();
                        let key = format!("{}", cli);
                        put(&cfg1, myck, &key, &last);
                        while done_clients1.load(Ordering::Relaxed) == 0 {
                            if (rng.gen::<u32>() % 1000) < 500 {
                                let nv = format!("x {} {} y", cli, j);
                                debug!("{}: client new append {}", cli, nv);
                                last = next_value(last, &nv);
                                append(&cfg1, myck, &key, &nv);
                                j += 1;
                            } else {
                                debug!("{}: client new get {:?}", cli, key);
                                let v = get(&cfg1, myck, &key);
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
                })
                .await
            })
        });

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
            partitioner_rx.recv().unwrap();
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
            let j = clnt_rx.recv().unwrap();
            if j < 10 {
                debug!(
                    "Warning: client {} managed to perform only {} put operations in 1 sec?",
                    i, j
                );
            }
            let key = format!("{}", i);
            debug!("Check {:?} for client {}", j, i);
            let v = get(&cfg, &ck, &key);
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

fn generic_test_linearizability(
    part: &str,
    nclients: usize,
    nservers: usize,
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
    title = format!("{}, linearizability checks ({})", title, part); // 3A or 3B

    let cfg = Arc::new(Config::new(nservers, unreliable, maxraftstate));

    cfg.begin(&title);

    let begin = Instant::now();
    let operations = Arc::new(Mutex::new(vec![]));

    let done_partitioner = Arc::new(AtomicUsize::new(0));
    let done_clients = Arc::new(AtomicUsize::new(0));
    let mut clnt_txs = vec![];
    let mut clnt_rxs = vec![];
    for _ in 0..nclients {
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
        let operations_ = operations.clone();
        cfg.net
            .spawn_poller(spawn_clients_and_wait(cfg.clone(), nclients, move || {
                let cfg1 = cfg_.clone();
                let clnt_txs1 = clnt_txs_.clone();
                let done_clients1 = done_clients_.clone();
                let operations1 = operations_.clone();
                move |cli, myck| {
                    // TODO: change the closure to a future.
                    let mut j = 0;
                    let mut rng = rand::thread_rng();
                    while done_clients1.load(Ordering::Relaxed) == 0 {
                        let key = format!("{}", rng.gen::<usize>() % nclients);
                        let nv = format!("x {} {} y", cli, j);

                        let start = begin.elapsed().as_nanos() as i64;
                        let (inp, out) = if rng.gen::<usize>() % 1000 < 500 {
                            append(&cfg1, myck, &key, &nv);
                            j += 1;
                            (
                                KvInput {
                                    op: Op::Append,
                                    key,
                                    value: nv,
                                },
                                KvOutput {
                                    value: "".to_string(),
                                },
                            )
                        } else if rng.gen::<usize>() % 1000 < 100 {
                            put(&cfg1, myck, &key, &nv);
                            j += 1;
                            (
                                KvInput {
                                    op: Op::Put,
                                    key,
                                    value: nv,
                                },
                                KvOutput {
                                    value: "".to_string(),
                                },
                            )
                        } else {
                            let v = get(&cfg1, myck, &key);
                            (
                                KvInput {
                                    op: Op::Get,
                                    key,
                                    value: "".to_string(),
                                },
                                KvOutput { value: v },
                            )
                        };

                        let end = begin.elapsed().as_nanos() as i64;
                        let op = Operation {
                            input: inp,
                            call: start,
                            output: out,
                            finish: end,
                        };
                        let mut data = operations1.lock().unwrap();
                        data.push(op);
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
            partitioner_rx.recv().unwrap();
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
            for i in 0..nservers {
                cfg.shutdown_server(i)
            }
            // Wait for a while for servers to shutdown, since
            // shutdown isn't a real crash and isn't instantaneous
            thread::sleep(RAFT_ELECTION_TIMEOUT);
            debug!("restart servers");
            // crash and re-start all
            for i in 0..nservers {
                cfg.start_server(i);
            }
            cfg.connect_all();
        }

        // wait for clients.
        for clnt_rx in &clnt_rxs {
            clnt_rx.recv().unwrap();
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

    if !check_operations_timeout(
        KvModel {},
        operations.lock().unwrap().clone(),
        LINEARIZABILITY_CHECK_TIMEOUT,
    ) {
        panic!("history is not linearizable");
    }
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
fn test_unreliable_one_key_3a() {
    let nservers = 3;
    let cfg = {
        let cfg = Config::new(nservers, true, None);
        cfg.begin("Test: concurrent append to same key, unreliable (3A)");
        Arc::new(cfg)
    };

    let all = cfg.all();
    let ck = cfg.make_client(&all);

    put(&cfg, &ck, "k", "");

    let cfg_ = cfg.clone();
    let nclient = 5;
    let upto = 10;
    block_on(async {
        spawn_clients_and_wait(cfg.clone(), nclient, move || {
            let cfg1 = cfg_.clone();
            move |me, myck| {
                for n in 0..upto {
                    append(&cfg1, myck, "k", &format!("x {} {} y", me, n));
                }
            }
        })
        .await
    });

    let counts = vec![upto; nclient];

    let vx = get(&cfg, &ck, "k");
    check_concurrent_appends(vx, &counts);

    cfg.check_timeout();
    cfg.end();
}

// Submit a request in the minority partition and check that the requests
// doesn't go through until the partition heals. The leader in the original
// network ends up in the minority partition.
#[test]
fn test_one_partition_3a() {
    let nservers = 5;
    let cfg = Config::new(nservers, false, None);

    let all = cfg.all();
    let ck = cfg.make_client(&all);

    put(&cfg, &ck, "1", "13");

    cfg.begin("Test: progress in majority (3A)");

    let (p1, p2) = cfg.make_partition();
    cfg.partition(&p1, &p2);

    // connect ckp1 to p1
    let ckp1 = cfg.make_client(&p1);
    // connect ckp2a to p2
    let ckp2a = cfg.make_client(&p2);
    let ckp2a_name = ckp2a.name.clone();
    // connect ckp2b to p2
    let ckp2b = cfg.make_client(&p2);
    let ckp2b_name = ckp2b.name.clone();

    put(&cfg, &ckp1, "1", "14");
    check(&cfg, &ckp1, "1", "14");

    cfg.end();

    let (done0_tx, done0_rx) = oneshot::channel::<&'static str>();
    let (done1_tx, done1_rx) = oneshot::channel::<&'static str>();

    cfg.begin("Test: no progress in minority (3A)");
    thread::spawn(move || {
        ckp2a.put("1".to_owned(), "15".to_owned());
        done0_tx
            .send("put")
            .map_err(|e| {
                warn!("done0 send failed: {:?}", e);
            })
            .unwrap();
    });
    let done0_rx = done0_rx.map(|op| {
        cfg.op();
        op
    });

    thread::spawn(move || {
        // different clerk in p2
        ckp2b.get("1".to_owned());
        done1_tx
            .send("get")
            .map_err(|e| {
                warn!("done0 send failed: {:?}", e);
            })
            .unwrap();
    });
    let done1_rx = done1_rx.map(|op| {
        cfg.op();
        op
    });

    let timeout = Delay::new(Duration::from_secs(1));

    let dones = block_on(
        future::select(timeout, future::select(done0_rx, done1_rx)).map(|res| match res {
            future::Either::Left((_, dones)) => dones,
            future::Either::Right((future::Either::Left((op, _)), _)) => {
                panic!("{} in minority completed", op.unwrap())
            }
            future::Either::Right((future::Either::Right((op, _)), _)) => {
                panic!("{} in minority completed", op.unwrap())
            }
        }),
    );

    check(&cfg, &ckp1, "1", "14");
    put(&cfg, &ckp1, "1", "16");
    check(&cfg, &ckp1, "1", "16");

    cfg.end();

    cfg.begin("Test: completion after heal (3A)");

    cfg.connect_all();
    cfg.connect_client_by_name(&ckp2a_name, &all);
    cfg.connect_client_by_name(&ckp2b_name, &all);

    thread::sleep(RAFT_ELECTION_TIMEOUT);

    let timeout = Delay::new(Duration::from_secs(3));
    let (timeout, next) = block_on(async {
        future::select(timeout, dones)
            .map(|res| match res {
                future::Either::Left(_) => panic!("put/get did not complete"),
                future::Either::Right((future::Either::Left((op, next)), timeout)) => {
                    info!("{} completes", op.unwrap());
                    (timeout, future::Either::Left(next))
                }
                future::Either::Right((future::Either::Right((op, next)), timeout)) => {
                    info!("{} completes", op.unwrap());
                    (timeout, future::Either::Right(next))
                }
            })
            .await
    });

    block_on(async {
        future::select(timeout, next)
            .map(|res| match res {
                future::Either::Left(_) => panic!("put/get did not complete"),
                future::Either::Right((op, _)) => info!("{} completes", op.unwrap()),
            })
            .await
    });

    check(&cfg, &ck, "1", "15");

    cfg.end();
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
fn test_persist_partition_unreliable_linearizable_3a() {
    // Test: unreliable net, restarts, partitions, linearizability checks (3A) ...
    generic_test_linearizability("3A", 15, 7, true, true, true, None)
}

// if one server falls behind, then rejoins, does it
// recover by using the InstallSnapshot RPC?
// also checks that majority discards committed log entries
// even if minority doesn't respond.
#[test]
fn test_snapshot_rpc_3b() {
    let nservers = 3;
    let maxraftstate = 1000;
    let cfg = Config::new(nservers, false, Some(maxraftstate));

    let all = cfg.all();
    let ck = cfg.make_client(&all);

    cfg.begin("Test: InstallSnapshot RPC (3B)");

    put(&cfg, &ck, "a", "A");
    check(&cfg, &ck, "a", "A");

    // a bunch of puts into the majority partition.
    cfg.partition(&[0, 1], &[2]);
    {
        let ck1 = cfg.make_client(&[0, 1]);
        for i in 0..50 {
            put(&cfg, &ck1, &format!("{}", i), &format!("{}", i));
        }
        thread::sleep(RAFT_ELECTION_TIMEOUT);
        put(&cfg, &ck1, "b", "B");
    }

    // check that the majority partition has thrown away
    // most of its log entries.
    if cfg.log_size() > 2 * maxraftstate {
        panic!(
            "logs were not trimmed ({} > 2*{})",
            cfg.log_size(),
            maxraftstate
        );
    }

    // now make group that requires participation of
    // lagging server, so that it has to catch up.
    cfg.partition(&[0, 2], &[1]);
    {
        let ck1 = cfg.make_client(&[0, 2]);
        put(&cfg, &ck1, "c", "C");
        put(&cfg, &ck1, "d", "D");
        check(&cfg, &ck1, "a", "A");
        check(&cfg, &ck1, "b", "B");
        check(&cfg, &ck1, "1", "1");
        check(&cfg, &ck1, "49", "49");
    }

    // now everybody
    cfg.partition(&[0, 1, 2], &[]);

    put(&cfg, &ck, "e", "E");
    check(&cfg, &ck, "c", "C");
    check(&cfg, &ck, "e", "E");
    check(&cfg, &ck, "1", "1");

    cfg.check_timeout();
    cfg.end();
}

// are the snapshots not too huge? 500 bytes is a generous bound for the
// operations we're doing here.
#[test]
fn test_snapshot_size_3b() {
    let nservers = 3;
    let maxraftstate = 1000;
    let maxsnapshotstate = 500;
    let cfg = Config::new(nservers, false, Some(maxraftstate));

    let all = cfg.all();
    let ck = cfg.make_client(&all);

    cfg.begin("Test: snapshot size is reasonable (3B)");

    for _ in 0..200 {
        put(&cfg, &ck, "x", "0");
        check(&cfg, &ck, "x", "0");
        put(&cfg, &ck, "x", "1");
        check(&cfg, &ck, "x", "1");
    }

    // check that servers have thrown away most of their log entries
    if cfg.log_size() > 2 * maxraftstate {
        panic!(
            "logs were not trimmed ({} > 2*{})",
            cfg.log_size(),
            maxraftstate,
        )
    }

    // check that the snapshots are not unreasonably large
    if cfg.snapshot_size() > maxsnapshotstate {
        panic!(
            "snapshot too large ({} > {})",
            cfg.snapshot_size(),
            maxsnapshotstate,
        )
    }

    cfg.check_timeout();
    cfg.end();
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

#[test]
fn test_snapshot_unreliable_recover_concurrent_partition_linearizable_3b() {
    // Test: unreliable net, restarts, partitions, snapshots, linearizability checks (3B) ...
    generic_test_linearizability("3B", 15, 7, true, true, true, Some(1000))
}
