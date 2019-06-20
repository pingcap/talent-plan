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
extern crate futures_cpupool;
extern crate futures_timer;
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

use futures::future;
use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::sync::oneshot;
use futures::{Async, Future, Poll, Stream};
use futures_cpupool::CpuPool;
use futures_timer::{Delay, Interval};
use hashbrown::HashMap;
use rand::Rng;

mod error;
#[macro_use]
mod macros;

pub use crate::error::{Error, Result};

static ID_ALLOC: AtomicUsize = AtomicUsize::new(0);

pub type RpcFuture<T> = Box<dyn Future<Item = T, Error = Error> + Send + 'static>;

pub type Handler = dyn Fn(&[u8]) -> RpcFuture<Vec<u8>>;

pub trait HandlerFactory: Sync + Send + 'static {
    fn handler(&self, name: &'static str) -> Box<Handler>;
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

    fn dispatch(&self, fq_name: &'static str, req: &[u8]) -> RpcFuture<Vec<u8>> {
        self.core.count.fetch_add(1, Ordering::Relaxed);
        let mut names = fq_name.split('.');
        let service_name = match names.next() {
            Some(n) => n,
            None => {
                return Box::new(future::result(Err(Error::Unimplemented(format!(
                    "unknown {}",
                    fq_name
                )))));
            }
        };
        let method_name = match names.next() {
            Some(n) => n,
            None => {
                return Box::new(future::result(Err(Error::Unimplemented(format!(
                    "unknown {}",
                    fq_name
                )))));
            }
        };
        if let Some(fact) = self.core.services.get(service_name) {
            let handle = fact.handler(method_name);
            handle(req)
        } else {
            Box::new(future::result(Err(Error::Unimplemented(format!(
                "unknown {}",
                fq_name
            )))))
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

    pub worker: CpuPool,
}

impl Client {
    pub fn call<Req, Rsp>(&self, fq_name: &'static str, req: &Req) -> RpcFuture<Rsp>
    where
        Req: labcodec::Message,
        Rsp: labcodec::Message + 'static,
    {
        let mut buf = vec![];
        if let Err(e) = labcodec::encode(req, &mut buf) {
            return Box::new(future::result(Err(Error::Encode(e))));
        }

        let (tx, rx) = oneshot::channel();
        let rpc = Rpc {
            client_name: self.name.clone(),
            fq_name,
            req: Some(buf),
            resp: Some(tx),
            hooks: self.hooks.clone(),
        };

        // Sends requets and waits responses.
        if self.sender.unbounded_send(rpc).is_err() {
            return Box::new(future::result(Err(Error::Stopped)));
        }
        Box::new(rx.then(|res| match res {
            Ok(Ok(resp)) => labcodec::decode(&resp).map_err(Error::Decode),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(Error::Recv(e)),
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
    poller: CpuPool,
    worker: CpuPool,
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
                poller: CpuPool::new(2),
                worker: CpuPool::new_num_cpus(),
                sender,
            }),
        };

        (net, incoming)
    }

    fn start(&self, incoming: UnboundedReceiver<Rpc>) {
        let net = self.clone();
        self.core
            .poller
            .spawn(incoming.for_each(move |mut rpc| {
                let resp = rpc.take_resp_sender().unwrap();
                net.core
                    .poller
                    .spawn(net.process_rpc(rpc).then(move |res| {
                        if let Err(e) = resp.send(res) {
                            error!("fail to send resp: {:?}", e);
                        }
                        Ok::<_, ()>(())
                    }))
                    .forget();
                Ok(())
            }))
            .forget();
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
            if enabled { "enabled" } else { "disbaled" }
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

    fn process_rpc(&self, rpc: Rpc) -> ProcessRpc {
        self.core.count.fetch_add(1, Ordering::Relaxed);
        let mut random = rand::thread_rng();
        let network = self.clone();
        let end_info = self.end_info(&rpc.client_name);
        debug!("{:?} process with {:?}", rpc, end_info);
        let EndInfo {
            enabled,
            reliable,
            long_reordering,
            server,
        } = end_info;

        if enabled && server.is_some() {
            let server = server.unwrap();
            let short_delay = if !reliable {
                // short delay
                let ms = random.gen::<u64>() % 27;
                Some(Delay::new(time::Duration::from_millis(ms)))
            } else {
                None
            };

            if !reliable && (random.gen::<u64>() % 1000) < 100 {
                // drop the request, return as if timeout
                return ProcessRpc {
                    state: Some(ProcessState::Timeout {
                        delay: short_delay.unwrap(),
                    }),
                    rpc,
                    network,
                    server: None,
                };
            }

            let drop_reply = !reliable && random.gen::<u64>() % 1000 < 100;
            let long_reordering = if long_reordering && random.gen_range(0, 900) < 600i32 {
                // delay the response for a while
                let upper_bound: u64 = 1 + random.gen_range(0, 2000);
                Some(200 + random.gen_range(0, upper_bound))
            } else {
                None
            };
            ProcessRpc {
                state: Some(ProcessState::Dispatch {
                    delay: short_delay,
                    drop_reply,
                    long_reordering,
                }),
                rpc,
                network,
                server: Some(server),
            }
        } else {
            // simulate no reply and eventual timeout.
            let ms = if self.core.long_delays.load(Ordering::Acquire) {
                // let Raft tests check that leader doesn't send
                // RPCs synchronously.
                random.gen::<u64>() % 7000
            } else {
                // many kv tests require the client to try each
                // server in fairly rapid succession.
                random.gen::<u64>() % 100
            };

            debug!("{:?} delay {}ms then timeout", rpc, ms);
            let delay = Delay::new(time::Duration::from_millis(ms));
            ProcessRpc {
                state: Some(ProcessState::Timeout { delay }),
                rpc,
                network,
                server: None,
            }
        }
    }

    /// Spawns a future to run on this net framework.
    pub fn spawn<F>(&self, f: F)
    where
        F: Future<Item = (), Error = ()> + Send + 'static,
    {
        self.core.worker.spawn(f).forget();
    }

    /// Spawns a future to run on this net framework.
    pub fn spawn_poller<F>(&self, f: F)
    where
        F: Future<Item = (), Error = ()> + Send + 'static,
    {
        self.core.poller.spawn(f).forget();
    }
}

struct ProcessRpc {
    state: Option<ProcessState>,

    rpc: Rpc,
    network: Network,
    server: Option<Server>,
}

impl fmt::Debug for ProcessRpc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ProcessRpc")
            .field("rpc", &self.rpc)
            .field("state", &self.state)
            .finish()
    }
}

enum ProcessState {
    Timeout {
        delay: Delay,
    },
    Dispatch {
        delay: Option<Delay>,
        drop_reply: bool,
        long_reordering: Option<u64>,
    },
    Ongoing {
        // I have to say it's ugly. :(
        res: Box<dyn Future<Item = Vec<u8>, Error = Error> + Send + 'static>,
        drop_reply: bool,
        long_reordering: Option<u64>,
    },
    Reordering {
        delay: Delay,
        resp: Option<Vec<u8>>,
    },
}

impl fmt::Debug for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProcessState::Timeout { .. } => write!(f, "ProcessState::Timeout"),
            ProcessState::Dispatch {
                ref delay,
                drop_reply,
                long_reordering,
            } => f
                .debug_struct("ProcessState::Dispatch")
                .field("delay", &delay.is_some())
                .field("drop_reply", &drop_reply)
                .field("long_reordering", &long_reordering)
                .finish(),
            ProcessState::Ongoing {
                drop_reply,
                long_reordering,
                ..
            } => f
                .debug_struct("ProcessState::Ongoing")
                .field("drop_reply", &drop_reply)
                .field("long_reordering", &long_reordering)
                .finish(),
            ProcessState::Reordering { .. } => write!(f, "ProcessState::Reordering"),
        }
    }
}

impl Future for ProcessRpc {
    type Item = Vec<u8>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Vec<u8>, Error> {
        let res = loop {
            let next;
            debug!("polling {:?}", self);
            match self
                .state
                .as_mut()
                .expect("cannot poll ProcessRpc after finish")
            {
                ProcessState::Timeout { ref mut delay } => {
                    try_ready!(delay.poll().map_err(|e| panic!("{:?}", e)));
                    break Err(Error::Timeout);
                }
                ProcessState::Dispatch {
                    ref mut delay,
                    drop_reply,
                    long_reordering,
                } => {
                    if let Some(ref mut delay) = *delay {
                        try_ready!(delay.poll().map_err(|e| panic!("{:?}", e)));
                    }
                    // We has finished the delay, take it out to prevent polling
                    // twice.
                    delay.take();

                    let fq_name = self.rpc.fq_name;
                    let req = self.rpc.req.take().unwrap();
                    let before_dispatch =
                        if let Some(hooks) = self.rpc.hooks.lock().unwrap().as_ref() {
                            hooks.before_dispatch(fq_name, &req)
                        } else {
                            Ok(())
                        };
                    let fut: Box<dyn Future<Item = Vec<u8>, Error = Error> + Send + 'static> =
                        if let Err(e) = before_dispatch {
                            Box::new(future::result(Err(e)))
                        } else {
                            // Execute the request (call the RPC handler)
                            // in a separate thread so that we can periodically check
                            // if the server has been killed and the RPC should get a
                            // failure reply.
                            //
                            // do not reply if DeleteServer() has been called, i.e.
                            // the server has been killed. this is needed to avoid
                            // situation in which a client gets a positive reply
                            // to an Append, but the server persisted the update
                            // into the old Persister. config.go is careful to call
                            // DeleteServer() before superseding the Persister.
                            let server = self.server.as_ref().unwrap();
                            let res = server.dispatch(fq_name, &req).select(ServerDead {
                                interval: Interval::new_at(
                                    time::Instant::now(),
                                    time::Duration::from_millis(100),
                                ),
                                net: self.network.clone(),
                                client_name: self.rpc.client_name.clone(),
                                server_name: server.core.name.clone(),
                                server_id: server.core.id,
                            });
                            let hooks: Arc<_> = self.rpc.hooks.clone();
                            let fut = res.then(move |res| {
                                let res = match res {
                                    // ServerDead never return Ok(_).
                                    Ok((resp, _)) => Ok(resp),
                                    // Server may be killed while we were waiting response.
                                    Err((e, _)) => Err(e),
                                };
                                let hooks = hooks.lock().unwrap();
                                if let Some(hooks) = hooks.as_ref() {
                                    hooks.after_dispatch(fq_name, res)
                                } else {
                                    res
                                }
                            });
                            Box::new(fut)
                        };
                    next = Some(ProcessState::Ongoing {
                        res: Box::new(fut),
                        drop_reply: *drop_reply,
                        long_reordering: *long_reordering,
                    });
                }
                ProcessState::Ongoing {
                    ref mut res,
                    drop_reply,
                    long_reordering,
                } => {
                    let resp = try_ready!(res.poll());
                    let server = self.server.as_ref().unwrap();
                    let is_server_dead = self.network.is_server_dead(
                        &self.rpc.client_name,
                        &server.core.name,
                        server.core.id,
                    );
                    if is_server_dead {
                        break Err(Error::Stopped);
                    } else if *drop_reply {
                        //  drop the reply, return as if timeout.
                        break Err(Error::Timeout);
                    } else if let Some(reordering) = long_reordering {
                        debug!("{:?} next long reordering {}ms", self.rpc, reordering);
                        next = Some(ProcessState::Reordering {
                            delay: Delay::new(time::Duration::from_millis(*reordering)),
                            resp: Some(resp),
                        });
                    } else {
                        break Ok(Async::Ready(resp));
                    }
                }
                ProcessState::Reordering {
                    ref mut delay,
                    ref mut resp,
                } => {
                    try_ready!(delay.poll().map_err(|e| panic!("{:?}", e)));
                    break Ok(Async::Ready(resp.take().unwrap()));
                }
            }
            if let Some(next) = next {
                self.state = Some(next);
            }
        };
        self.state.take();
        res
    }
}

/// A future checks if the specified server killed.
///
/// It will never return Ok but Err when the server is killed.
struct ServerDead {
    interval: Interval,
    net: Network,
    client_name: String,
    server_name: String,
    server_id: usize,
}

impl Future for ServerDead {
    type Item = Vec<u8>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Vec<u8>, Error> {
        loop {
            try_ready!(self.interval.poll().map_err(|e| panic!("{:?}", e)));
            if self
                .net
                .is_server_dead(&self.client_name, &self.server_name, self.server_id)
            {
                debug!("{:?} is dead", self.server_name);
                return Err(Error::Stopped);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{mpsc, Mutex, Once};
    use std::thread;

    use futures::sync::oneshot::Canceled;

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
    impl Junk for JunkService {
        fn handler2(&self, args: JunkArgs) -> RpcFuture<JunkReply> {
            self.inner.lock().unwrap().log2.push(args.x);
            Box::new(future::result(Ok(JunkReply {
                x: format!("handler2-{}", args.x),
            })))
        }
        fn handler3(&self, args: JunkArgs) -> RpcFuture<JunkReply> {
            Box::new(
                Delay::new(time::Duration::from_secs(20))
                    .and_then(move |_| {
                        future::result(Ok(JunkReply {
                            x: format!("handler3-{}", -args.x),
                        }))
                    })
                    .map_err(|e| panic!("{:?}", e)),
            )
        }
        fn handler4(&self, _: JunkArgs) -> RpcFuture<JunkReply> {
            Box::new(future::result(Ok(JunkReply {
                x: "pointer".to_owned(),
            })))
        }
    }

    fn init_logger() {
        static LOGGER_INIT: Once = Once::new();
        LOGGER_INIT.call_once(env_logger::init);
    }

    #[test]
    fn test_service_dispatch() {
        init_logger();

        let mut builder = ServerBuilder::new("test".to_owned());
        let junk = JunkService::new();
        add_service(junk.clone(), &mut builder).unwrap();
        let prev_len = builder.services.len();
        add_service(junk, &mut builder).unwrap_err();
        assert_eq!(builder.services.len(), prev_len);
        let server = builder.build();

        let buf = server.dispatch("junk.handler4", &[]).wait().unwrap();
        let rsp = labcodec::decode(&buf).unwrap();
        assert_eq!(
            JunkReply {
                x: "pointer".to_owned(),
            },
            rsp,
        );

        server
            .dispatch("junk.handler4", b"bad message")
            .wait()
            .unwrap_err();

        server.dispatch("badjunk.handler4", &[]).wait().unwrap_err();

        server.dispatch("junk.badhandler", &[]).wait().unwrap_err();
    }

    #[test]
    fn test_network_client_rpc() {
        init_logger();

        let mut builder = ServerBuilder::new("test".to_owned());
        let junk = JunkService::new();
        add_service(junk, &mut builder).unwrap();
        let server = builder.build();

        let (net, incoming) = Network::create();
        net.add_server(server);

        let client = JunkClient::new(net.create_client("test_client".to_owned()));
        let (tx, rx) = mpsc::channel();
        client.spawn(client.handler4(&JunkArgs { x: 777 }).then(move |reply| {
            tx.send(reply).unwrap();
            Ok(())
        }));
        let (mut rpc, incoming) = match incoming.into_future().wait() {
            Ok((Some(rpc), s)) => (rpc, s),
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
        client.spawn(client.handler4(&JunkArgs { x: 777 }).then(move |reply| {
            tx.send(reply).unwrap();
            Ok(())
        }));
        let (rpc, incoming) = match incoming.into_future().wait() {
            Ok((Some(rpc), s)) => (rpc, s),
            _ => panic!("unexpected error"),
        };
        drop(rpc.resp);
        assert_eq!(rx.recv().unwrap(), Err(Error::Recv(Canceled)));

        drop(incoming);
        assert_eq!(
            client.handler4(&JunkArgs::default()).wait(),
            Err(Error::Stopped)
        );
    }

    #[test]
    fn test_basic() {
        init_logger();

        let (net, _, _) = junk_suit();

        let client = JunkClient::new(net.create_client("test_client".to_owned()));
        net.connect("test_client", "test_server");
        net.enable("test_client", true);

        let rsp = client.handler4(&JunkArgs::default()).wait().unwrap();
        assert_eq!(
            JunkReply {
                x: "pointer".to_owned(),
            },
            rsp,
        );
    }

    // does net.Enable(endname, false) really disconnect a client?
    #[test]
    fn test_disconnect() {
        init_logger();

        let (net, _, _) = junk_suit();

        let client = JunkClient::new(net.create_client("test_client".to_owned()));
        net.connect("test_client", "test_server");

        client.handler4(&JunkArgs::default()).wait().unwrap_err();

        net.enable("test_client", true);
        let rsp = client.handler4(&JunkArgs::default()).wait().unwrap();

        assert_eq!(
            JunkReply {
                x: "pointer".to_owned(),
            },
            rsp,
        );
    }

    // test net.GetCount()
    #[test]
    fn test_count() {
        init_logger();

        let (net, _, _) = junk_suit();

        let client = JunkClient::new(net.create_client("test_client".to_owned()));
        net.connect("test_client", "test_server");
        net.enable("test_client", true);

        for i in 0..=16 {
            let reply = client.handler2(&JunkArgs { x: i }).wait().unwrap();
            assert_eq!(reply.x, format!("handler2-{}", i));
        }

        assert_eq!(net.count("test_server"), 17,);
    }

    // test RPCs from concurrent Clients
    #[test]
    fn test_concurrent_many() {
        init_logger();

        let (net, server, _) = junk_suit();
        let server_name = server.name();

        let pool = futures_cpupool::CpuPool::new_num_cpus();
        let (tx, rx) = mpsc::channel::<usize>();

        let nclients = 20usize;
        let nrpcs = 10usize;
        for i in 0..nclients {
            let net = net.clone();
            let sender = tx.clone();
            let server_name_ = server_name.to_string();

            pool.spawn_fn(move || {
                let mut n = 0;
                let client_name = format!("client-{}", i);
                let client = JunkClient::new(net.create_client(client_name.clone()));
                net.enable(&client_name, true);
                net.connect(&client_name, &server_name_);

                for j in 0..nrpcs {
                    let x = (i * 100 + j) as i64;
                    let reply = client.handler2(&JunkArgs { x }).wait().unwrap();
                    assert_eq!(reply.x, format!("handler2-{}", x));
                    n += 1;
                }

                sender.send(n)
            })
            .forget();
        }

        let mut total = 0;
        for _ in 0..nclients {
            total += rx.recv().unwrap();
        }
        assert_eq!(total, nrpcs * nclients);
        let n = net.count(&server_name);
        assert_eq!(n, total);
    }

    fn junk_suit() -> (Network, Server, JunkService) {
        let net = Network::new();
        let server_name = "test_server".to_owned();
        let mut builder = ServerBuilder::new(server_name.clone());
        let junk_server = JunkService::new();
        add_service(junk_server.clone(), &mut builder).unwrap();
        let server = builder.build();
        net.add_server(server.clone());
        (net, server, junk_server)
    }

    #[test]
    fn test_unreliable() {
        init_logger();

        let (net, server, _) = junk_suit();
        let server_name = server.name();
        net.set_reliable(false);

        let pool = futures_cpupool::CpuPool::new_num_cpus();
        let (tx, rx) = mpsc::channel::<usize>();
        let nclients = 300;
        for i in 0..nclients {
            let sender = tx.clone();
            let mut n = 0;
            let server_name_ = server_name.to_owned();
            let net = net.clone();

            pool.spawn_fn(move || {
                let client_name = format!("client-{}", i);
                let client = JunkClient::new(net.create_client(client_name.clone()));
                net.enable(&client_name, true);
                net.connect(&client_name, &server_name_);

                let x = i * 100;
                if let Ok(reply) = client.handler2(&JunkArgs { x }).wait() {
                    assert_eq!(reply.x, format!("handler2-{}", x));
                    n += 1;
                }
                sender.send(n)
            })
            .forget();
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
    }

    // test concurrent RPCs from a single Client
    #[test]
    fn test_concurrent_one() {
        init_logger();

        let (net, server, junk_server) = junk_suit();
        let server_name = server.name();

        let pool = futures_cpupool::CpuPool::new_num_cpus();
        let (tx, rx) = mpsc::channel::<usize>();
        let nrpcs = 20;
        for i in 0..20 {
            let sender = tx.clone();
            let client_name = format!("client-{}", i);
            let client = JunkClient::new(net.create_client(client_name.clone()));
            net.enable(&client_name, true);
            net.connect(&client_name, server_name);

            pool.spawn_fn(move || {
                let mut n = 0;
                let x = i + 100;
                let reply = client.handler2(&JunkArgs { x }).wait().unwrap();
                assert_eq!(reply.x, format!("handler2-{}", x));
                n += 1;
                sender.send(n)
            })
            .forget();
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
    }

    // regression: an RPC that's delayed during Enabled=false
    // should not delay subsequent RPCs (e.g. after Enabled=true).
    #[test]
    fn test_regression1() {
        init_logger();

        let (net, server, junk_server) = junk_suit();
        let server_name = server.name();

        let client_name = "client";
        let client = JunkClient::new(net.create_client(client_name.to_owned()));
        net.connect(client_name, server_name);

        // start some RPCs while the Client is disabled.
        // they'll be delayed.
        net.enable(client_name, false);

        let pool = futures_cpupool::CpuPool::new_num_cpus();
        let (tx, rx) = mpsc::channel::<bool>();
        let nrpcs = 20;
        for i in 0..20 {
            let sender = tx.clone();
            let client_ = client.clone();
            pool.spawn_fn(move || {
                let x = i + 100;
                // this call ought to return false.
                let _ = client_.handler2(&JunkArgs { x });
                sender.send(true)
            })
            .forget();
        }

        // FIXME: have to sleep 300ms to pass the test
        //        in my computer (i5-4200U, 4 threads).
        thread::sleep(time::Duration::from_millis(100 * 3));

        let t0 = time::Instant::now();
        net.enable(client_name, true);
        let x = 99;
        let reply = client.handler2(&JunkArgs { x }).wait().unwrap();
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
        client.spawn(client.handler3(&JunkArgs { x: 99 }).then(move |reply| {
            tx.send(reply).unwrap();
            Ok(())
        }));
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
        let reply = client.handler2(&JunkArgs { x: i }).wait().unwrap();
        assert_eq!(reply.x, format!("handler2-{}", i));
        hook.drop_req.store(true, Ordering::Relaxed);
        assert_eq!(
            client.handler2(&JunkArgs { x: i }).wait().unwrap_err(),
            Error::Other("reqhook".to_owned())
        );
        hook.drop_req.store(false, Ordering::Relaxed);
        hook.drop_resp.store(true, Ordering::Relaxed);
        assert_eq!(
            client.handler2(&JunkArgs { x: i }).wait().unwrap_err(),
            Error::Other("resphook".to_owned())
        );
        hook.drop_resp.store(false, Ordering::Relaxed);
        client.handler2(&JunkArgs { x: i }).wait().unwrap();
        assert_eq!(reply.x, format!("handler2-{}", i));
    }

    #[bench]
    fn bench_rpc(b: &mut test::Bencher) {
        let (net, server, _junk_server) = junk_suit();
        let server_name = server.name();
        let client_name = "client";
        let client = JunkClient::new(net.create_client(client_name.to_owned()));
        net.connect(client_name, server_name);
        net.enable(client_name, true);

        b.iter(|| {
            client.handler2(&JunkArgs { x: 111 }).wait().unwrap();
        });
        // i5-4200U, 21 microseconds per RPC
    }
}
