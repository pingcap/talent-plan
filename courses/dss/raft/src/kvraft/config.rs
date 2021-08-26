use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rand::seq::SliceRandom;

use crate::kvraft::errors::{Error, Result};
use crate::kvraft::{client, server};
use crate::proto::kvraftpb::*;
use crate::proto::raftpb::*;
use crate::raft;
use crate::raft::persister::*;

static ID: AtomicUsize = AtomicUsize::new(300_000);

fn uniqstring() -> String {
    format!("{}", ID.fetch_add(1, Ordering::Relaxed))
}

struct Servers {
    kvservers: Vec<Option<server::Node>>,
    saved: Vec<Arc<SimplePersister>>,
    endnames: Vec<Vec<String>>,
}

fn init_logger() {
    use std::sync::Once;
    static LOGGER_INIT: Once = Once::new();
    LOGGER_INIT.call_once(env_logger::init);
}

pub struct Config {
    pub net: labrpc::Network,
    pub n: usize,
    servers: Mutex<Servers>,
    clerks: Mutex<HashMap<String, Vec<String>>>,
    next_client_id: AtomicUsize,
    maxraftstate: Option<usize>,

    // time at which the Config was created.
    start: Instant,

    // begin()/end() statistics
    // time at which test_test.go called cfg.begin()
    t0: Mutex<Instant>,
    // rpc_total() at start of test
    rpcs0: AtomicUsize,
    // number of agreements
    ops: AtomicUsize,
}

impl Config {
    pub fn new(n: usize, unreliable: bool, maxraftstate: Option<usize>) -> Config {
        init_logger();

        let servers = Servers {
            kvservers: vec![None; n],
            saved: (0..n).map(|_| Arc::new(SimplePersister::new())).collect(),
            endnames: vec![vec![String::new(); n]; n],
        };
        let cfg = Config {
            n,
            net: labrpc::Network::new(),
            servers: Mutex::new(servers),
            clerks: Mutex::new(HashMap::new()),
            // client ids start 1000 above the highest serverid,
            next_client_id: AtomicUsize::new(n + 1000),
            maxraftstate,
            start: Instant::now(),
            t0: Mutex::new(Instant::now()),
            rpcs0: AtomicUsize::new(0),
            ops: AtomicUsize::new(0),
        };

        // create a full set of KV servers.
        for i in 0..cfg.n {
            cfg.start_server(i);
        }

        cfg.connect_all();

        cfg.net.set_reliable(!unreliable);

        cfg
    }

    pub fn op(&self) {
        self.ops.fetch_add(1, Ordering::Relaxed);
    }

    fn rpc_total(&self) -> usize {
        self.net.total_count()
    }

    pub fn check_timeout(&self) {
        // enforce a two minute real-time limit on each test
        if self.start.elapsed() > Duration::from_secs(120) {
            panic!("test took longer than 120 seconds");
        }
    }

    /// Maximum log size across all servers
    pub fn log_size(&self) -> usize {
        let servers = self.servers.lock().unwrap();
        let mut logsize = 0;
        for save in &servers.saved {
            let n = save.raft_state().len();
            if n > logsize {
                logsize = n;
            }
        }
        logsize
    }

    /// Maximum snapshot size across all servers
    pub fn snapshot_size(&self) -> usize {
        let mut snapshotsize = 0;
        let servers = self.servers.lock().unwrap();
        for save in &servers.saved {
            let n = save.snapshot().len();
            if n > snapshotsize {
                snapshotsize = n;
            }
        }
        snapshotsize
    }

    /// Attach server i to servers listed in to
    fn connect(&self, i: usize, to: &[usize], servers: &Servers) {
        debug!("connect peer {} to {:?}", i, to);
        // outgoing socket files
        for j in to {
            let endname = &servers.endnames[i][*j];
            self.net.enable(endname, true);
        }

        // incoming socket files
        for j in to {
            let endname = &servers.endnames[*j][i];
            self.net.enable(endname, true);
        }
    }

    /// Detach server i from the servers listed in from
    fn disconnect(&self, i: usize, from: &[usize], servers: &Servers) {
        debug!("disconnect peer {} from {:?}", i, from);
        // outgoing socket files
        for j in from {
            if !servers.endnames[i].is_empty() {
                let endname = &servers.endnames[i][*j];
                self.net.enable(endname, false);
            }
        }

        // incoming socket files
        for j in from {
            if !servers.endnames[*j].is_empty() {
                let endname = &servers.endnames[*j][i];
                self.net.enable(endname, false);
            }
        }
    }

    pub fn all(&self) -> Vec<usize> {
        (0..self.n).collect()
    }

    pub fn connect_all(&self) {
        let servers = self.servers.lock().unwrap();
        for i in 0..self.n {
            self.connect(i, &self.all(), &*servers);
        }
    }

    /// Sets up 2 partitions with connectivity between servers in each  partition.
    pub fn partition(&self, p1: &[usize], p2: &[usize]) {
        debug!("partition servers into: {:?} {:?}", p1, p2);
        let servers = self.servers.lock().unwrap();
        for i in p1 {
            self.disconnect(*i, p2, &*servers);
            self.connect(*i, p1, &*servers);
        }
        for i in p2 {
            self.disconnect(*i, p1, &*servers);
            self.connect(*i, p2, &*servers);
        }
    }

    // Create a clerk with clerk specific server names.
    // Give it connections to all of the servers, but for
    // now enable only connections to servers in to[].
    pub fn make_client(&self, to: &[usize]) -> client::Clerk {
        // a fresh set of ClientEnds.
        let mut ends = Vec::with_capacity(self.n);
        let mut endnames = Vec::with_capacity(self.n);
        for j in 0..self.n {
            let name = uniqstring();
            endnames.push(name.clone());
            let cli = self.net.create_client(name.clone());
            ends.push(KvClient::new(cli));
            self.net.connect(&name, &format!("{}", j));
        }

        ends.shuffle(&mut rand::thread_rng());
        let ck_name = uniqstring();
        let ck = client::Clerk::new(ck_name.clone(), ends);
        self.clerks.lock().unwrap().insert(ck_name, endnames);
        self.next_client_id.fetch_add(1, Ordering::Relaxed);
        self.connect_client(&ck, to);
        ck
    }

    pub fn delete_client(&self, ck: &client::Clerk) {
        self.clerks.lock().unwrap().remove(&ck.name);
    }

    pub fn connect_client(&self, ck: &client::Clerk, to: &[usize]) {
        self.connect_client_by_name(&ck.name, to);
    }

    pub fn connect_client_by_name(&self, ck_name: &str, to: &[usize]) {
        debug!("connect_client {:?} to {:?}", ck_name, to);
        let clerks = self.clerks.lock().unwrap();
        let endnames = &clerks[ck_name];
        for j in to {
            let s = &endnames[*j];
            self.net.enable(s, true);
        }
    }

    /// Shutdown a server by isolating it
    pub fn shutdown_server(&self, i: usize) {
        let mut servers = self.servers.lock().unwrap();
        self.disconnect(i, &self.all(), &*servers);

        // disable client connections to the server.
        // it's important to do this before creating
        // the new Persister in saved[i], to avoid
        // the possibility of the server returning a
        // positive reply to an Append but persisting
        // the result in the superseded Persister.
        self.net.delete_server(&format!("{}", i));

        // a fresh persister, in case old instance
        // continues to update the Persister.
        // but copy old persister's content so that we always
        // pass Make() the last persisted state.
        let p = raft::persister::SimplePersister::new();
        p.save_state_and_snapshot(servers.saved[i].raft_state(), servers.saved[i].snapshot());
        servers.saved[i] = Arc::new(p);

        if let Some(kv) = servers.kvservers[i].take() {
            kv.kill();
        }
    }

    /// Start a server i.
    /// If restart servers, first call shutdown_server
    pub fn start_server(&self, i: usize) {
        // a fresh set of outgoing ClientEnd names.
        let mut servers = self.servers.lock().unwrap();
        servers.endnames[i] = (0..self.n).map(|_| uniqstring()).collect();

        // a fresh set of ClientEnds.
        let mut ends = Vec::with_capacity(self.n);
        for (j, name) in servers.endnames[i].iter().enumerate() {
            let cli = self.net.create_client(name.clone());
            ends.push(RaftClient::new(cli));
            self.net.connect(name, &format!("{}", j));
        }

        // a fresh persister, so old instance doesn't overwrite
        // new instance's persisted state.
        // give the fresh persister a copy of the old persister's
        // state, so that the spec is that we pass StartKVServer()
        // the last persisted state.
        let sp = raft::persister::SimplePersister::new();
        sp.save_state_and_snapshot(servers.saved[i].raft_state(), servers.saved[i].snapshot());
        let p = Arc::new(sp);
        servers.saved[i] = p.clone();

        let kv = server::KvServer::new(ends, i, Box::new(p), self.maxraftstate);
        let rf_node = kv.rf.clone();
        let kv_node = server::Node::new(kv);
        servers.kvservers[i] = Some(kv_node.clone());

        let mut builder = labrpc::ServerBuilder::new(format!("{}", i));
        add_raft_service(rf_node, &mut builder).unwrap();
        add_kv_service(kv_node, &mut builder).unwrap();
        let srv = builder.build();
        self.net.add_server(srv);
    }

    pub fn leader(&self) -> Result<usize> {
        let servers = self.servers.lock().unwrap();
        for (i, kv) in servers.kvservers.iter().enumerate() {
            if let Some(kv) = kv {
                if kv.is_leader() {
                    return Ok(i);
                }
            }
        }
        Err(Error::NoLeader)
    }

    /// Partition servers into 2 groups and put current leader in minority
    pub fn make_partition(&self) -> (Vec<usize>, Vec<usize>) {
        let l = self.leader().unwrap_or(0);
        let mut p1 = Vec::with_capacity(self.n / 2 + 1);
        let mut p2 = Vec::with_capacity(self.n / 2);
        for i in 0..self.n {
            if i != l {
                if p1.len() < self.n / 2 + 1 {
                    p1.push(i);
                } else {
                    p2.push(i);
                }
            }
        }
        p2.push(l);
        (p1, p2)
    }

    /// Start a Test.
    /// print the Test message.
    /// e.g. cfg.begin("Test (2B): RPC counts aren't too high")
    pub fn begin(&self, description: &str) {
        println!(); // Force the log starts at a new line.
        info!("{} ...", description);
        *self.t0.lock().unwrap() = Instant::now();
        self.rpcs0.store(self.rpc_total(), Ordering::Relaxed);
        self.ops.store(0, Ordering::Relaxed);
    }

    /// End a Test -- the fact that we got here means there
    /// was no failure.
    /// print the Passed message,
    /// and some performance numbers.
    pub fn end(&self) {
        self.check_timeout();

        // real time
        let t = self.t0.lock().unwrap().elapsed();
        // number of Raft peers
        let npeers = self.n;
        // number of RPC sends
        let nrpc = self.rpc_total() - self.rpcs0.load(Ordering::Relaxed);
        // number of clerk get/put/append calls
        let nops = self.ops.load(Ordering::Relaxed);

        info!("  ... Passed --");
        info!("  {:?}  {} {} {}", t, npeers, nrpc, nops);
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        let servers = self.servers.lock().unwrap();
        for s in servers.kvservers.iter().flatten() {
            s.kill();
        }
    }
}
