#![allow(unused_attributes)]
#![allow(clippy::new_without_default)]
#![allow(clippy::if_same_then_else)]
#![feature(test)]
#[cfg(test)]
extern crate test;

extern crate bytes;
#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;
extern crate hashbrown;
extern crate labcodec;
extern crate prost;
extern crate rand;

#[cfg(test)]
#[macro_use]
extern crate prost_derive;
#[cfg(test)]
extern crate env_logger;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::{fmt, time};

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot;
use futures::executor::ThreadPool;
use futures::future;
use futures::future::{BoxFuture, FutureExt};
use futures::prelude::*;
use futures_timer::Delay;
use hashbrown::HashMap;
use rand::Rng;

mod error;
#[macro_use]
mod macros;

pub use crate::error::{Error, Result};
use time::Duration;

static ID_ALLOC: AtomicUsize = AtomicUsize::new(0);

pub trait HandlerFactory: Sync + Send + 'static {
    fn handler(&self, name: &'static str) -> Box<dyn FnOnce(&[u8]) -> BoxFuture<Result<Vec<u8>>>>;
}

pub struct ServerBuilder {
    name: String,
    // Service name -> service methods
    services: HashMap<&'static str, Box<dyn HandlerFactory>>,
}

impl ServerBuilder {
    pub fn new(name: String) -> ServerBuilder {
        ServerBuilder {
            name,
            services: HashMap::new(),
        }
    }

    pub fn add_service(
        &mut self,
        service_name: &'static str,
        fact: Box<dyn HandlerFactory>,
    ) -> Result<()> {
        match self.services.entry(service_name) {
            hashbrown::hash_map::Entry::Occupied(_) => Err(Error::Other(format!(
                "{} has already registered",
                service_name
            ))),
            hashbrown::hash_map::Entry::Vacant(entry) => {
                entry.insert(fact);
                Ok(())
            }
        }
    }

    pub fn build(self) -> Server {
        Server {
            core: Arc::new(ServerCore {
                name: self.name,
                services: self.services,
                id: ID_ALLOC.fetch_add(1, Ordering::Relaxed),
                count: AtomicUsize::new(0),
            }),
        }
    }
}

struct ServerCore {
    name: String,
    id: usize,

    services: HashMap<&'static str, Box<dyn HandlerFactory>>,
    count: AtomicUsize,
}

#[derive(Clone)]
pub struct Server {
    core: Arc<ServerCore>,
}

impl Server {
    pub fn count(&self) -> usize {
        self.core.count.load(Ordering::Relaxed)
    }

    pub fn name(&self) -> &str {
        &self.core.name
    }

    fn dispatch<'a>(&self, fq_name: &'static str, req: &'a [u8]) -> BoxFuture<'a, Result<Vec<u8>>> {
        self.core.count.fetch_add(1, Ordering::Relaxed);
        let mut names = fq_name.split('.');
        let service_name = match names.next() {
            Some(n) => n,
            None => {
                return Box::pin(future::err(Error::Unimplemented(format!(
                    "unknown {}",
                    fq_name
                ))));
            }
        };
        let method_name = match names.next() {
            Some(n) => n,
            None => {
                return Box::pin(future::err(Error::Unimplemented(format!(
                    "unknown {}",
                    fq_name
                ))));
            }
        };
        if let Some(fact) = self.core.services.get(service_name) {
            let handle = fact.handler(method_name);
            handle(req)
        } else {
            Box::pin(future::err(Error::Unimplemented(format!(
                "unknown {}",
                fq_name
            ))))
        }
    }
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Server")
            .field("name", &self.core.name)
            .field("id", &self.core.id)
            .finish()
    }
}

pub struct Rpc {
    client_name: String,
    fq_name: &'static str,
    req: Option<Vec<u8>>,
    resp: Option<oneshot::Sender<Result<Vec<u8>>>>,
    hooks: Arc<Mutex<Option<Arc<dyn RpcHooks>>>>,
}

impl Rpc {
    fn take_resp_sender(&mut self) -> Option<oneshot::Sender<Result<Vec<u8>>>> {
        self.resp.take()
    }
}

impl fmt::Debug for Rpc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Rpc")
            .field("client_name", &self.client_name)
            .field("fq_name", &self.fq_name)
            .finish()
    }
}

pub trait RpcHooks: Sync + Send + 'static {
    fn before_dispatch(&self, fq_name: &str, req: &[u8]) -> Result<()>;
    fn after_dispatch(&self, fq_name: &str, resp: Result<Vec<u8>>) -> Result<Vec<u8>>;
}

#[derive(Clone)]
pub struct Client {
    // this end-point's name
    name: String,
    // copy of Network.sender
    sender: UnboundedSender<Rpc>,
    hooks: Arc<Mutex<Option<Arc<dyn RpcHooks>>>>,

    pub worker: ThreadPool,
}

impl Client {
    pub fn call<Req, Rsp>(&self, fq_name: &'static str, req: &Req) -> BoxFuture<Result<Rsp>>
    where
        Req: labcodec::Message,
        Rsp: labcodec::Message + 'static,
    {
        let mut buf = vec![];
        if let Err(e) = labcodec::encode(req, &mut buf) {
            return Box::pin(future::err(Error::Encode(e)));
        }

        let (tx, rx) = oneshot::channel();
        let rpc = Rpc {
            client_name: self.name.clone(),
            fq_name,
            req: Some(buf),
            resp: Some(tx),
            hooks: self.hooks.clone(),
        };
        // Sends requests and waits responses.
        if self.sender.unbounded_send(rpc).is_err() {
            return Box::pin(future::err(Error::Stopped));
        }
        Box::pin(rx.then(|res| async {
            match res {
                Ok(Ok(resp)) => labcodec::decode(&resp).map_err(Error::Decode),
                Ok(Err(e)) => Err(e),
                Err(e) => Err(Error::Recv(e)),
            }
        }))
    }

    pub fn set_hooks(&self, hooks: Arc<dyn RpcHooks>) {
        *self.hooks.lock().unwrap() = Some(hooks);
    }

    pub fn clear_hooks(&self) {
        *self.hooks.lock().unwrap() = None;
    }
}

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

struct Core {
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
    core: Arc<Core>,
}

impl Network {
    pub fn new() -> Network {
        let (net, incoming) = Network::create();
        net.start(incoming);
        net
    }

    fn create() -> (Network, UnboundedReceiver<Rpc>) {
        let (sender, incoming) = unbounded();
        let net = Network {
            core: Arc::new(Core {
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
        let net = self.clone();
        self.core.poller.spawn_ok(async move {
            while let Some(mut rpc) = incoming.next().await {
                let resp = rpc.take_resp_sender().unwrap();
                let n = net.clone();
                net.core.poller.spawn_ok(async move {
                    let res = n.process_rpc(rpc).await;
                    if let Err(e) = resp.send(res) {
                        error!("fail to send resp: {:?}", e);
                    }
                });
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
                    Some(rand::thread_rng().gen::<u64>() % 27)
                } else {
                    None
                };

                if !reliable && (rand::thread_rng().gen::<u64>() % 1000) < 100 {
                    // drop the request, return as if timeout
                    Delay::new(Duration::from_millis(short_delay.unwrap())).await;
                    return Err(Error::Timeout);
                }

                let drop_reply = !reliable && rand::thread_rng().gen::<u64>() % 1000 < 100;
                let long_reordering =
                    if long_reordering && rand::thread_rng().gen_range(0, 900) < 600i32 {
                        // delay the response for a while
                        let upper_bound: u64 = 1 + rand::thread_rng().gen_range(0, 2000);
                        Some(200 + rand::thread_rng().gen_range(0, upper_bound))
                    } else {
                        None
                    };

                process_rpc(
                    short_delay,
                    drop_reply,
                    long_reordering,
                    rpc,
                    network,
                    Some(server),
                )
                .await
            }
            _ => {
                // simulate no reply and eventual timeout.
                let ms = if self.core.long_delays.load(Ordering::Acquire) {
                    // let Raft tests check that leader doesn't send
                    // RPCs synchronously.
                    rand::thread_rng().gen::<u64>() % 7000
                } else {
                    // many kv tests require the client to try each
                    // server in fairly rapid succession.
                    rand::thread_rng().gen::<u64>() % 100
                };
                debug!("{:?} delay {}ms then timeout", rpc, ms);
                Delay::new(time::Duration::from_millis(ms)).await;
                return Err(Error::Timeout);
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

/// Checks if the specified server killed.
///
/// It will return when the server is killed.
async fn server_dead(
    interval: Duration,
    net: Network,
    client_name: String,
    server_name: String,
    server_id: usize,
) {
    loop {
        Delay::new(interval).await;
        if net.is_server_dead(&client_name, &server_name, server_id) {
            debug!("{:?} is dead", server_name);
            return;
        }
    }
}

async fn process_rpc(
    delay: Option<u64>,
    drop_reply: bool,
    long_reordering: Option<u64>,

    mut rpc: Rpc,
    network: Network,
    server: Option<Server>,
) -> Result<Vec<u8>> {
    if let Some(delay) = delay {
        Delay::new(Duration::from_millis(delay)).await;
    }
    let fq_name = rpc.fq_name;
    let req = rpc.req.take().unwrap();
    if let Some(hooks) = rpc.hooks.lock().unwrap().as_ref() {
        hooks.before_dispatch(fq_name, &req)?;
    };
    let server = server.unwrap();
    let res = select! {
        res = server.dispatch(fq_name, &req).fuse() => res,
        _ = server_dead(
            Duration::from_millis(100),
            network.clone(),
            rpc.client_name.clone(),
            server.core.name.clone(),
            server.core.id
        ).fuse() => Err(Error::Stopped),
    };
    let resp = if let Some(hooks) = rpc.hooks.lock().unwrap().as_ref() {
        hooks.after_dispatch(fq_name, res)?
    } else {
        res?
    };
    if network.is_server_dead(&rpc.client_name, &server.core.name, server.core.id) {
        Err(Error::Stopped)
    } else if drop_reply {
        //  drop the reply, return as if timeout.
        Err(Error::Timeout)
    } else if let Some(reordering) = long_reordering {
        debug!("{:?} next long reordering {}ms", rpc, reordering);
        Delay::new(time::Duration::from_millis(reordering)).await;
        Ok(resp)
    } else {
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{mpsc, Mutex, Once};
    use std::thread;

    use futures::channel::oneshot::Canceled;
    use futures::executor::block_on;
    use async_trait::async_trait;

    use super::*;

    service! {
        /// A simple test-purpose service.
        service junk {
            /// Doc comments.
            rpc handler2(JunkArgs) returns (JunkReply);
            rpc handler3(JunkArgs) returns (JunkReply);
            rpc handler4(JunkArgs) returns (JunkReply);
        }
    }
    use self::tests::junk::{add_service, Client as JunkClient, Service as Junk};

    // Hand-written protobuf messages.
    #[derive(Clone, PartialEq, Message)]
    pub struct JunkArgs {
        #[prost(int64, tag = "1")]
        pub x: i64,
    }
    #[derive(Clone, PartialEq, Message)]
    pub struct JunkReply {
        #[prost(string, tag = "1")]
        pub x: String,
    }

    #[derive(Default)]
    struct JunkInner {
        log2: Vec<i64>,
    }
    #[derive(Clone)]
    struct JunkService {
        inner: Arc<Mutex<JunkInner>>,
    }
    impl JunkService {
        fn new() -> JunkService {
            JunkService {
                inner: Arc::default(),
            }
        }
    }
    #[async_trait]
    impl Junk for JunkService {
        async fn handler2(&self, args: JunkArgs) -> Result<JunkReply> {
            self.inner.lock().unwrap().log2.push(args.x);
            Ok(JunkReply {
                x: format!("handler2-{}", args.x),
            })
        }
        async fn handler3(&self, args: JunkArgs) -> Result<JunkReply> {
            Delay::new(time::Duration::from_secs(20)).await;
            Ok(JunkReply {
                x: format!("handler3-{}", -args.x),
            })
        }
        async fn handler4(&self, _: JunkArgs) -> Result<JunkReply> {
            Ok(JunkReply {
                x: "pointer".to_owned(),
            })
        }
    }

    fn init_logger() {
        static LOGGER_INIT: Once = Once::new();
        LOGGER_INIT.call_once(env_logger::init);
    }

    #[test]
    fn test_service_dispatch() {
        block_on(async {
            init_logger();

            let mut builder = ServerBuilder::new("test".to_owned());
            let junk = JunkService::new();
            add_service(junk.clone(), &mut builder).unwrap();
            let prev_len = builder.services.len();
            add_service(junk, &mut builder).unwrap_err();
            assert_eq!(builder.services.len(), prev_len);
            let server = builder.build();

            let buf = server.dispatch("junk.handler4", &[]).await.unwrap();
            let rsp = labcodec::decode(&buf).unwrap();
            assert_eq!(
                JunkReply {
                    x: "pointer".to_owned(),
                },
                rsp,
            );

            server
                .dispatch("junk.handler4", b"bad message")
                .await
                .unwrap_err();

            server.dispatch("badjunk.handler4", &[]).await.unwrap_err();

            server.dispatch("junk.badhandler", &[]).await.unwrap_err();
        });
    }

    #[test]
    fn test_network_client_rpc() {
        block_on(async {
            init_logger();

            let mut builder = ServerBuilder::new("test".to_owned());
            let junk = JunkService::new();
            add_service(junk, &mut builder).unwrap();
            let server = builder.build();

            let (net, incoming) = Network::create();
            net.add_server(server);

            let client = JunkClient::new(net.create_client("test_client".to_owned()));
            let (tx, rx) = mpsc::channel();
            let cli = client.clone();
            client.spawn(async move {
                let reply = cli.handler4(&JunkArgs { x: 777 }).await;
                tx.send(reply).unwrap();
            });
            let (mut rpc, incoming) = match incoming.into_future().await {
                (Some(rpc), s) => (rpc, s),
                _ => panic!("unexpected error"),
            };
            let reply = JunkReply {
                x: "boom!!!".to_owned(),
            };
            let mut buf = vec![];
            labcodec::encode(&reply, &mut buf).unwrap();
            let resp = rpc.take_resp_sender().unwrap();
            resp.send(Ok(buf)).unwrap();
            assert_eq!(rpc.client_name, "test_client");
            assert_eq!(rpc.fq_name, "junk.handler4");
            assert!(!rpc.req.as_ref().unwrap().is_empty());
            assert_eq!(rx.recv().unwrap(), Ok(reply));

            let (tx, rx) = mpsc::channel();
            let cli = client.clone();
            client.spawn(async move {
                let reply = cli.handler4(&JunkArgs { x: 777 }).await;
                tx.send(reply).unwrap();
            });
            let (rpc, incoming) = match incoming.into_future().await {
                (Some(rpc), s) => (rpc, s),
                _ => panic!("unexpected error"),
            };
            drop(rpc.resp);
            assert_eq!(rx.recv().unwrap(), Err(Error::Recv(Canceled)));

            drop(incoming);
            assert_eq!(
                client.handler4(&JunkArgs::default()).await,
                Err(Error::Stopped)
            );
        });
    }

    #[test]
    fn test_basic() {
        block_on(async {
            init_logger();

            let (net, _, _) = junk_suit();

            let client = JunkClient::new(net.create_client("test_client".to_owned()));
            net.connect("test_client", "test_server");
            net.enable("test_client", true);

            let rsp = client.handler4(&JunkArgs::default()).await.unwrap();
            assert_eq!(
                JunkReply {
                    x: "pointer".to_owned(),
                },
                rsp,
            );
        });
    }

    // does net.Enable(endname, false) really disconnect a client?
    #[test]
    fn test_disconnect() {
        block_on(async {
            init_logger();

            let (net, _, _) = junk_suit();

            let client = JunkClient::new(net.create_client("test_client".to_owned()));
            net.connect("test_client", "test_server");

            client.handler4(&JunkArgs::default()).await.unwrap_err();

            net.enable("test_client", true);
            let rsp = client.handler4(&JunkArgs::default()).await.unwrap();

            assert_eq!(
                JunkReply {
                    x: "pointer".to_owned(),
                },
                rsp,
            );
        });
    }

    // test net.GetCount()
    #[test]
    fn test_count() {
        block_on(async {
            init_logger();

            let (net, _, _) = junk_suit();

            let client = JunkClient::new(net.create_client("test_client".to_owned()));
            net.connect("test_client", "test_server");
            net.enable("test_client", true);

            for i in 0..=16 {
                let reply = client.handler2(&JunkArgs { x: i }).await.unwrap();
                assert_eq!(reply.x, format!("handler2-{}", i));
            }

            assert_eq!(net.count("test_server"), 17,);
        });
    }

    // test RPCs from concurrent Clients
    #[test]
    fn test_concurrent_many() {
        block_on(async {
            init_logger();

            let (net, server, _) = junk_suit();
            let server_name = server.name();

            let pool = ThreadPool::new().unwrap();
            let (tx, rx) = mpsc::channel::<usize>();

            let nclients = 20usize;
            let nrpcs = 10usize;
            for i in 0..nclients {
                let net = net.clone();
                let sender = tx.clone();
                let server_name_ = server_name.to_string();

                pool.spawn_ok(async move {
                    let mut n = 0;
                    let client_name = format!("client-{}", i);
                    let client = JunkClient::new(net.create_client(client_name.clone()));
                    net.enable(&client_name, true);
                    net.connect(&client_name, &server_name_);

                    for j in 0..nrpcs {
                        let x = (i * 100 + j) as i64;
                        let reply = client.handler2(&JunkArgs { x }).await.unwrap();
                        assert_eq!(reply.x, format!("handler2-{}", x));
                        n += 1;
                    }

                    sender.send(n).unwrap();
                });
            }

            let mut total = 0;
            for _ in 0..nclients {
                total += rx.recv().unwrap();
            }
            assert_eq!(total, nrpcs * nclients);
            let n = net.count(&server_name);
            assert_eq!(n, total);
        });
    }

    fn junk_suit() -> (Network, Server, JunkService) {
        let net = Network::new();
        let server_name = "test_server".to_owned();
        let mut builder = ServerBuilder::new(server_name);
        let junk_server = JunkService::new();
        add_service(junk_server.clone(), &mut builder).unwrap();
        let server = builder.build();
        net.add_server(server.clone());
        (net, server, junk_server)
    }

    #[test]
    fn test_unreliable() {
        block_on(async {
            init_logger();

            let (net, server, _) = junk_suit();
            let server_name = server.name();
            net.set_reliable(false);

            let pool = ThreadPool::new().unwrap();
            let (tx, rx) = mpsc::channel::<usize>();
            let nclients = 300;
            for i in 0..nclients {
                let sender = tx.clone();
                let mut n = 0;
                let server_name_ = server_name.to_owned();
                let net = net.clone();

                pool.spawn_ok(async move {
                    let client_name = format!("client-{}", i);
                    let client = JunkClient::new(net.create_client(client_name.clone()));
                    net.enable(&client_name, true);
                    net.connect(&client_name, &server_name_);

                    let x = i * 100;
                    if let Ok(reply) = client.handler2(&JunkArgs { x }).await {
                        assert_eq!(reply.x, format!("handler2-{}", x));
                        n += 1;
                    }
                    sender.send(n).unwrap();
                });
            }
            let mut total = 0;
            for _ in 0..nclients {
                total += rx.recv().unwrap();
            }
            assert!(
                !(total == nclients as _ || total == 0),
                "all RPCs succeeded despite unreliable total {}, nclients {}",
                total,
                nclients
            );
        });
    }

    // test concurrent RPCs from a single Client
    #[test]
    fn test_concurrent_one() {
        block_on(async {
            init_logger();

            let (net, server, junk_server) = junk_suit();
            let server_name = server.name();

            let pool = ThreadPool::new().unwrap();
            let (tx, rx) = mpsc::channel::<usize>();
            let nrpcs = 20;
            for i in 0..20 {
                let sender = tx.clone();
                let client_name = format!("client-{}", i);
                let client = JunkClient::new(net.create_client(client_name.clone()));
                net.enable(&client_name, true);
                net.connect(&client_name, server_name);

                pool.spawn_ok(async move {
                    let mut n = 0;
                    let x = i + 100;
                    let reply = client.handler2(&JunkArgs { x }).await.unwrap();
                    assert_eq!(reply.x, format!("handler2-{}", x));
                    n += 1;
                    sender.send(n).unwrap();
                });
            }

            let mut total = 0;
            for _ in 0..nrpcs {
                total += rx.recv().unwrap();
            }
            assert!(
                total == nrpcs,
                "wrong number of RPCs completed, got {}, expected {}",
                total,
                nrpcs
            );

            assert_eq!(
                junk_server.inner.lock().unwrap().log2.len(),
                nrpcs,
                "wrong number of RPCs delivered"
            );

            let n = net.count(server.name());
            assert_eq!(n, total, "wrong count() {}, expected {}", n, total);
        });
    }

    // regression: an RPC that's delayed during Enabled=false
    // should not delay subsequent RPCs (e.g. after Enabled=true).
    #[test]
    fn test_regression1() {
        block_on(async {
            init_logger();

            let (net, server, junk_server) = junk_suit();
            let server_name = server.name();

            let client_name = "client";
            let client = JunkClient::new(net.create_client(client_name.to_owned()));
            net.connect(client_name, server_name);

            // start some RPCs while the Client is disabled.
            // they'll be delayed.
            net.enable(client_name, false);

            let pool = ThreadPool::new().unwrap();
            let (tx, rx) = mpsc::channel::<bool>();
            let nrpcs = 20;
            for i in 0..20 {
                let sender = tx.clone();
                let client_ = client.clone();
                pool.spawn_ok(async move {
                    let x = i + 100;
                    // this call ought to return false.
                    let _ = client_.handler2(&JunkArgs { x });
                    sender.send(true).unwrap();
                });
            }

            // FIXME: have to sleep 300ms to pass the test
            //        in my computer (i5-4200U, 4 threads).
            thread::sleep(time::Duration::from_millis(100 * 3));

            let t0 = time::Instant::now();
            net.enable(client_name, true);
            let x = 99;
            let reply = client.handler2(&JunkArgs { x }).await.unwrap();
            assert_eq!(reply.x, format!("handler2-{}", x));
            let dur = t0.elapsed();
            assert!(
                dur < time::Duration::from_millis(30),
                "RPC took too long ({:?}) after Enable",
                dur
            );

            for _ in 0..nrpcs {
                rx.recv().unwrap();
            }

            let len = junk_server.inner.lock().unwrap().log2.len();
            assert!(
                len == 1,
                "wrong number ({}) of RPCs delivered, expected 1",
                len
            );

            let n = net.count(server.name());
            assert!(n == 1, "wrong count() {}, expected 1", n);
        });
    }

    // if an RPC is stuck in a server, and the server
    // is killed with DeleteServer(), does the RPC
    // get un-stuck?
    #[test]
    fn test_killed() {
        init_logger();

        let (net, server, _junk_server) = junk_suit();
        let server_name = server.name();

        let client_name = "client";
        let client = JunkClient::new(net.create_client(client_name.to_owned()));
        net.connect(client_name, &server_name);
        net.enable(client_name, true);
        let (tx, rx) = mpsc::channel();
        let cli = client.clone();
        client.spawn(async move {
            let reply = cli.handler3(&JunkArgs { x: 99 }).await;
            tx.send(reply).unwrap();
        });
        thread::sleep(time::Duration::from_secs(1));
        rx.recv_timeout(time::Duration::from_millis(100))
            .unwrap_err();

        net.delete_server(server_name);
        let reply = rx.recv_timeout(time::Duration::from_millis(100)).unwrap();
        assert_eq!(reply, Err(Error::Stopped));
    }

    struct Hooks {
        drop_req: AtomicBool,
        drop_resp: AtomicBool,
    }
    impl RpcHooks for Hooks {
        fn before_dispatch(&self, _: &str, _: &[u8]) -> Result<()> {
            if self.drop_req.load(Ordering::Relaxed) {
                Err(Error::Other("reqhook".to_owned()))
            } else {
                Ok(())
            }
        }
        fn after_dispatch(&self, _: &str, resp: Result<Vec<u8>>) -> Result<Vec<u8>> {
            if self.drop_resp.load(Ordering::Relaxed) {
                Err(Error::Other("resphook".to_owned()))
            } else {
                resp
            }
        }
    }

    #[test]
    fn test_rpc_hooks() {
        block_on(async {
            init_logger();
            let (net, _, _) = junk_suit();

            let raw_cli = net.create_client("test_client".to_owned());
            let hook = Arc::new(Hooks {
                drop_req: AtomicBool::new(false),
                drop_resp: AtomicBool::new(false),
            });
            raw_cli.set_hooks(hook.clone());

            let client = JunkClient::new(raw_cli);
            net.connect("test_client", "test_server");
            net.enable("test_client", true);

            let i = 100;
            let reply = client.handler2(&JunkArgs { x: i }).await.unwrap();
            assert_eq!(reply.x, format!("handler2-{}", i));
            hook.drop_req.store(true, Ordering::Relaxed);
            assert_eq!(
                client.handler2(&JunkArgs { x: i }).await.unwrap_err(),
                Error::Other("reqhook".to_owned())
            );
            hook.drop_req.store(false, Ordering::Relaxed);
            hook.drop_resp.store(true, Ordering::Relaxed);
            assert_eq!(
                client.handler2(&JunkArgs { x: i }).await.unwrap_err(),
                Error::Other("resphook".to_owned())
            );
            hook.drop_resp.store(false, Ordering::Relaxed);
            client.handler2(&JunkArgs { x: i }).await.unwrap();
            assert_eq!(reply.x, format!("handler2-{}", i));
        });
    }

    #[bench]
    fn bench_rpc(b: &mut test::Bencher) {
        let (net, server, _junk_server) = junk_suit();
        let server_name = server.name();
        let client_name = "client";
        let client = JunkClient::new(net.create_client(client_name.to_owned()));
        net.connect(client_name, server_name);
        net.enable(client_name, true);

        b.iter(|| block_on(client.handler2(&JunkArgs { x: 111 })));
        // i5-4200U, 21 microseconds per RPC
    }
}
