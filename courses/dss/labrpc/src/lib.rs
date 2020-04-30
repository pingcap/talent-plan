#![allow(clippy::new_without_default)]

mod client;
mod error;
#[macro_use]
mod macros;
mod network;
mod server;

pub use self::client::{Client, Rpc, RpcHooks};
pub use self::error::{Error, Result};
pub use self::network::Network;
pub use self::server::{Handler, HandlerFactory, RpcFuture, Server, ServerBuilder};

#[cfg(test)]
pub mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc, Mutex, Once};
    use std::thread;
    use std::time::{Duration, Instant};

    use futures::channel::oneshot::Canceled;
    use futures::executor::{block_on, ThreadPool};
    use futures::stream::StreamExt;
    use futures_timer::Delay;
    use prost_derive::Message;

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
    use junk::{add_service, Client as JunkClient, Service as Junk};

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
    #[async_trait::async_trait]
    impl Junk for JunkService {
        async fn handler2(&self, args: JunkArgs) -> Result<JunkReply> {
            self.inner.lock().unwrap().log2.push(args.x);
            Ok(JunkReply {
                x: format!("handler2-{}", args.x),
            })
        }
        async fn handler3(&self, args: JunkArgs) -> Result<JunkReply> {
            Delay::new(Duration::from_secs(20)).await;
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
        init_logger();

        let mut builder = ServerBuilder::new("test".to_owned());
        let junk = JunkService::new();
        add_service(junk.clone(), &mut builder).unwrap();
        let prev_len = builder.services.len();
        add_service(junk, &mut builder).unwrap_err();
        assert_eq!(builder.services.len(), prev_len);
        let server = builder.build();

        let buf = block_on(async { server.dispatch("junk.handler4", &[]).await.unwrap() });
        let rsp = labcodec::decode(&buf).unwrap();
        assert_eq!(
            JunkReply {
                x: "pointer".to_owned(),
            },
            rsp,
        );

        block_on(async {
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
        let (mut rpc, incoming) = block_on(async {
            match incoming.into_future().await {
                (Some(rpc), s) => (rpc, s),
                _ => panic!("unexpected error"),
            }
        });
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
        let (rpc, incoming) = block_on(async {
            match incoming.into_future().await {
                (Some(rpc), s) => (rpc, s),
                _ => panic!("unexpected error"),
            }
        });
        drop(rpc.resp);
        assert_eq!(rx.recv().unwrap(), Err(Error::Recv(Canceled)));

        drop(incoming);
        assert_eq!(
            block_on(async { client.handler4(&JunkArgs::default()).await }),
            Err(Error::Stopped)
        );
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
    fn test_basic() {
        init_logger();

        let (net, _, _) = junk_suit();

        let client = JunkClient::new(net.create_client("test_client".to_owned()));
        net.connect("test_client", "test_server");
        net.enable("test_client", true);

        let rsp = block_on(async { client.handler4(&JunkArgs::default()).await.unwrap() });
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

        block_on(async { client.handler4(&JunkArgs::default()).await.unwrap_err() });

        net.enable("test_client", true);
        let rsp = block_on(async { client.handler4(&JunkArgs::default()).await.unwrap() });

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
            let reply = block_on(async { client.handler2(&JunkArgs { x: i }).await.unwrap() });
            assert_eq!(reply.x, format!("handler2-{}", i));
        }

        assert_eq!(net.count("test_server"), 17);
    }

    // test RPCs from concurrent Clients
    #[test]
    fn test_concurrent_many() {
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
            let server_name = server_name.to_string();

            pool.spawn_ok(async move {
                let mut n = 0;
                let client_name = format!("client-{}", i);
                let client = JunkClient::new(net.create_client(client_name.clone()));
                net.enable(&client_name, true);
                net.connect(&client_name, &server_name);

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
        let n = net.count(server_name);
        assert_eq!(n, total);
    }

    #[test]
    fn test_unreliable() {
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
            let server_name = server_name.to_owned();
            let net = net.clone();

            pool.spawn_ok(async move {
                let client_name = format!("client-{}", i);
                let client = JunkClient::new(net.create_client(client_name.clone()));
                net.enable(&client_name, true);
                net.connect(&client_name, &server_name);

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
    }

    // test concurrent RPCs from a single Client
    #[test]
    fn test_concurrent_one() {
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
                sender.send(n).unwrap()
            });
        }

        let mut total = 0;
        for _ in 0..nrpcs {
            total += rx.recv().unwrap();
        }
        assert_eq!(
            total, nrpcs,
            "wrong number of RPCs completed, got {}, expected {}",
            total, nrpcs
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

        let pool = ThreadPool::new().unwrap();
        let (tx, rx) = mpsc::channel::<bool>();
        let nrpcs = 20;
        for i in 0..20 {
            let sender = tx.clone();
            let cli = client.clone();
            pool.spawn_ok(async move {
                let x = i + 100;
                // this call ought to return false.
                let _ = cli.handler2(&JunkArgs { x });
                sender.send(true).unwrap();
            });
        }

        // FIXME: have to sleep 300ms to pass the test
        //        in my computer (i5-4200U, 4 threads).
        thread::sleep(Duration::from_millis(100 * 3));

        let t0 = Instant::now();
        net.enable(client_name, true);
        let x = 99;
        let reply = block_on(async { client.handler2(&JunkArgs { x }).await.unwrap() });
        assert_eq!(reply.x, format!("handler2-{}", x));
        let dur = t0.elapsed();
        assert!(
            dur < Duration::from_millis(30),
            "RPC took too long ({:?}) after Enable",
            dur
        );

        for _ in 0..nrpcs {
            rx.recv().unwrap();
        }

        let len = junk_server.inner.lock().unwrap().log2.len();
        assert_eq!(
            len, 1,
            "wrong number ({}) of RPCs delivered, expected 1",
            len
        );

        let n = net.count(server.name());
        assert_eq!(n, 1, "wrong count() {}, expected 1", n);
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
        net.connect(client_name, server_name);
        net.enable(client_name, true);
        let (tx, rx) = mpsc::channel();
        let cli = client.clone();
        client.spawn(async move {
            let reply = cli.handler3(&JunkArgs { x: 99 }).await;
            tx.send(reply).unwrap();
        });
        thread::sleep(Duration::from_secs(1));
        rx.recv_timeout(Duration::from_millis(100)).unwrap_err();

        net.delete_server(server_name);
        let reply = rx.recv_timeout(Duration::from_millis(100)).unwrap();
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
        let reply = block_on(async { client.handler2(&JunkArgs { x: i }).await.unwrap() });
        assert_eq!(reply.x, format!("handler2-{}", i));
        hook.drop_req.store(true, Ordering::Relaxed);
        assert_eq!(
            block_on(async { client.handler2(&JunkArgs { x: i }).await.unwrap_err() }),
            Error::Other("reqhook".to_owned())
        );
        hook.drop_req.store(false, Ordering::Relaxed);
        hook.drop_resp.store(true, Ordering::Relaxed);
        assert_eq!(
            block_on(async { client.handler2(&JunkArgs { x: i }).await.unwrap_err() }),
            Error::Other("resphook".to_owned())
        );
        hook.drop_resp.store(false, Ordering::Relaxed);
        block_on(async { client.handler2(&JunkArgs { x: i }).await.unwrap() });
        assert_eq!(reply.x, format!("handler2-{}", i));
    }
}
