use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::executor::ThreadPool;
use futures::future::FutureExt;
use futures::select;
use futures::stream::StreamExt;
use futures_timer::Delay;
use log::{debug, error};
use rand::{thread_rng, Rng};

use crate::client::{Client, Rpc};
use crate::error::{Error, Result};
use crate::server::Server;

#[derive(Debug)]
struct EndInfo {
    enabled: bool,
    reliable: bool,
    long_reordering: bool,
    server: Option<Server>,
}

struct Endpoints {
    // by client name
    enabled: HashMap<String, bool>,
    // servers, by name
    servers: HashMap<String, Option<Server>>,
    // client_name -> server_name
    connections: HashMap<String, Option<String>>,
}

struct NetworkCore {
    reliable: AtomicBool,
    // pause a long time on send on disabled connection
    long_delays: AtomicBool,
    // sometimes delay replies a long time
    long_reordering: AtomicBool,
    endpoints: Mutex<Endpoints>,
    count: AtomicUsize,
    sender: UnboundedSender<Rpc>,
    poller: ThreadPool,
    worker: ThreadPool,
}

#[derive(Clone)]
pub struct Network {
    core: Arc<NetworkCore>,
}

impl Network {
    pub fn new() -> Network {
        let (net, incoming) = Network::create();
        net.start(incoming);
        net
    }

    pub fn create() -> (Network, UnboundedReceiver<Rpc>) {
        let (sender, incoming) = unbounded();
        let net = Network {
            core: Arc::new(NetworkCore {
                reliable: AtomicBool::new(true),
                long_delays: AtomicBool::new(false),
                long_reordering: AtomicBool::new(false),
                endpoints: Mutex::new(Endpoints {
                    enabled: HashMap::new(),
                    servers: HashMap::new(),
                    connections: HashMap::new(),
                }),
                count: AtomicUsize::new(0),
                poller: ThreadPool::builder().pool_size(2).create().unwrap(),
                worker: ThreadPool::new().unwrap(),
                sender,
            }),
        };

        (net, incoming)
    }

    fn start(&self, mut incoming: UnboundedReceiver<Rpc>) {
        let network = self.clone();
        self.core.poller.spawn_ok(async move {
            while let Some(mut rpc) = incoming.next().await {
                let resp = rpc.take_resp_sender().unwrap();
                let net = network.clone();
                network.core.poller.spawn_ok(async move {
                    let res = net.process_rpc(rpc).await;
                    if let Err(e) = resp.send(res) {
                        error!("fail to send resp: {:?}", e);
                    }
                })
            }
        });
    }

    pub fn add_server(&self, server: Server) {
        let mut eps = self.core.endpoints.lock().unwrap();
        eps.servers.insert(server.core.name.clone(), Some(server));
    }

    pub fn delete_server(&self, name: &str) {
        let mut eps = self.core.endpoints.lock().unwrap();
        if let Some(s) = eps.servers.get_mut(name) {
            *s = None;
        }
    }

    pub fn create_client(&self, name: String) -> Client {
        let sender = self.core.sender.clone();
        let mut eps = self.core.endpoints.lock().unwrap();
        eps.enabled.insert(name.clone(), false);
        eps.connections.insert(name.clone(), None);
        Client {
            name,
            sender,
            worker: self.core.worker.clone(),
            hooks: Arc::new(Mutex::new(None)),
        }
    }

    /// Connects a Client to a server.
    /// a Client can only be connected once in its lifetime.
    pub fn connect(&self, client_name: &str, server_name: &str) {
        let mut eps = self.core.endpoints.lock().unwrap();
        eps.connections
            .insert(client_name.to_owned(), Some(server_name.to_owned()));
    }

    /// Enable/disable a Client.
    pub fn enable(&self, client_name: &str, enabled: bool) {
        debug!(
            "client {} is {}",
            client_name,
            if enabled { "enabled" } else { "disabled" }
        );
        let mut eps = self.core.endpoints.lock().unwrap();
        eps.enabled.insert(client_name.to_owned(), enabled);
    }

    pub fn set_reliable(&self, yes: bool) {
        self.core.reliable.store(yes, Ordering::Release);
    }

    pub fn set_long_reordering(&self, yes: bool) {
        self.core.long_reordering.store(yes, Ordering::Release);
    }

    pub fn set_long_delays(&self, yes: bool) {
        self.core.long_delays.store(yes, Ordering::Release);
    }

    pub fn count(&self, server_name: &str) -> usize {
        let eps = self.core.endpoints.lock().unwrap();
        eps.servers[server_name].as_ref().unwrap().count()
    }

    pub fn total_count(&self) -> usize {
        self.core.count.load(Ordering::Relaxed)
    }

    fn end_info(&self, client_name: &str) -> EndInfo {
        let eps = self.core.endpoints.lock().unwrap();
        let mut server = None;
        if let Some(Some(server_name)) = eps.connections.get(client_name) {
            server = eps.servers[server_name].clone();
        }
        EndInfo {
            enabled: eps.enabled[client_name],
            reliable: self.core.reliable.load(Ordering::Acquire),
            long_reordering: self.core.long_reordering.load(Ordering::Acquire),
            server,
        }
    }

    fn is_server_dead(&self, client_name: &str, server_name: &str, server_id: usize) -> bool {
        let eps = self.core.endpoints.lock().unwrap();
        !eps.enabled[client_name]
            || eps.servers.get(server_name).map_or(true, |o| {
                o.as_ref().map(|s| s.core.id != server_id).unwrap_or(true)
            })
    }

    async fn process_rpc(&self, rpc: Rpc) -> Result<Vec<u8>> {
        self.core.count.fetch_add(1, Ordering::Relaxed);
        let network = self.clone();
        let end_info = self.end_info(&rpc.client_name);
        debug!("{:?} process with {:?}", rpc, end_info);
        let EndInfo {
            enabled,
            reliable,
            long_reordering,
            server,
        } = end_info;

        match (enabled, server) {
            (true, Some(server)) => {
                let short_delay = if !reliable {
                    // short delay
                    let ms = thread_rng().gen::<u64>() % 27;
                    Some(ms)
                } else {
                    None
                };

                if !reliable && (thread_rng().gen::<u64>() % 1000) < 100 {
                    // drop the request, return as if timeout
                    Delay::new(Duration::from_secs(short_delay.unwrap())).await;
                    return Err(Error::Timeout);
                }

                let drop_reply = !reliable && thread_rng().gen::<u64>() % 1000 < 100;
                let long_reordering = if long_reordering && thread_rng().gen_range(0, 900) < 600i32
                {
                    // delay the response for a while
                    let upper_bound: u64 = 1 + thread_rng().gen_range(0, 2000);
                    Some(200 + thread_rng().gen_range(0, upper_bound))
                } else {
                    None
                };

                // Dispatch
                process_rpc(
                    short_delay,
                    drop_reply,
                    long_reordering,
                    rpc,
                    network,
                    server,
                )
                .await
            }
            _ => {
                // simulate no reply and eventual timeout.
                let ms = if self.core.long_delays.load(Ordering::Acquire) {
                    // let Raft tests check that leader doesn't send
                    // RPCs synchronously.
                    thread_rng().gen::<u64>() % 7000
                } else {
                    // many kv tests require the client to try each
                    // server in fairly rapid succession.
                    thread_rng().gen::<u64>() % 100
                };

                debug!("{:?} delay {}ms then timeout", rpc, ms);
                Delay::new(Duration::from_millis(ms)).await;
                Err(Error::Timeout)
            }
        }
    }

    /// Spawns a future to run on this net framework.
    pub fn spawn<F>(&self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.core.worker.spawn_ok(f);
    }

    /// Spawns a future to run on this net framework.
    pub fn spawn_poller<F>(&self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.core.poller.spawn_ok(f);
    }
}

async fn process_rpc(
    mut delay: Option<u64>,
    drop_reply: bool,
    long_reordering: Option<u64>,
    mut rpc: Rpc,
    network: Network,
    server: Server,
) -> Result<Vec<u8>> {
    // Dispatch ===============================================================
    if let Some(delay) = delay {
        Delay::new(Duration::from_millis(delay)).await;
    }
    // We has finished the delay, take it out to prevent polling
    // twice.
    delay.take();

    let fq_name = rpc.fq_name;
    let req = rpc.req.take().unwrap();
    if let Some(hooks) = rpc.hooks.lock().unwrap().as_ref() {
        hooks.before_dispatch(fq_name, &req)?;
    }

    // Execute the request (call the RPC handler) in a separate thread so that
    // we can periodically check if the server has been killed and the RPC
    // should get a failure reply.
    //
    // do not reply if DeleteServer() has been called, i.e. the server has been killed.
    // this is needed to avoid situation in which a client gets a positive reply
    // to an Append, but the server persisted the update into the old Persister.
    // config.go is careful to call DeleteServer() before superseding the Persister.
    let resp = select! {
        res = server.dispatch(fq_name, &req).fuse() => res,
        _ = server_dead(
            Duration::from_millis(100),
            network.clone(),
            &rpc.client_name,
            &server.core.name,
            server.core.id,
        ).fuse() => Err(Error::Stopped),
    };

    let resp = if let Some(hooks) = rpc.hooks.lock().unwrap().as_ref() {
        hooks.after_dispatch(fq_name, resp)?
    } else {
        resp?
    };

    // Ongoing ================================================================
    let client_name = &rpc.client_name;
    let server_name = &server.core.name;
    let server_id = server.core.id;
    if network.is_server_dead(client_name, server_name, server_id) {
        return Err(Error::Stopped);
    }
    if drop_reply {
        // drop the reply, return as if timeout.
        return Err(Error::Timeout);
    }

    // Reordering =============================================================
    if let Some(reordering) = long_reordering {
        debug!("{:?} next long reordering {}ms", rpc, reordering);
        Delay::new(Duration::from_millis(reordering)).await;
        Ok(resp)
    } else {
        Ok(resp)
    }
}

/// Checks if the specified server killed.
///
/// It will return when the server is killed.
async fn server_dead(
    interval: Duration,
    net: Network,
    client_name: &str,
    server_name: &str,
    server_id: usize,
) {
    loop {
        Delay::new(interval).await;
        if net.is_server_dead(client_name, server_name, server_id) {
            debug!("{:?} is dead", server_name);
            return;
        }
    }
}
