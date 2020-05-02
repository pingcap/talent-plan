use std::sync::{Arc, Mutex};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use futures::executor::block_on;
use prost_derive::Message;

use labrpc::{service, Network, Result, Server, ServerBuilder};

service! {
    /// A simple bench-purpose service.
    service bench {
        /// Doc comments.
        rpc handler(BenchArgs) returns (BenchReply);
    }
}
use bench::{add_service, Client as BenchClient, Service};

// Hand-written protobuf messages.
#[derive(Clone, PartialEq, Message)]
pub struct BenchArgs {
    #[prost(int64, tag = "1")]
    pub x: i64,
}

#[derive(Clone, PartialEq, Message)]
pub struct BenchReply {
    #[prost(string, tag = "1")]
    pub x: String,
}

#[derive(Default)]
struct BenchInner {
    log2: Vec<i64>,
}
#[derive(Clone)]
pub struct BenchService {
    inner: Arc<Mutex<BenchInner>>,
}
impl BenchService {
    fn new() -> BenchService {
        BenchService {
            inner: Arc::default(),
        }
    }
}

#[async_trait::async_trait]
impl Service for BenchService {
    async fn handler(&self, args: BenchArgs) -> Result<BenchReply> {
        self.inner.lock().unwrap().log2.push(args.x);
        Ok(BenchReply {
            x: format!("handler-{}", args.x),
        })
    }
}

fn bench_suit() -> (Network, Server, BenchService) {
    let net = Network::new();
    let server_name = "test_server".to_owned();
    let mut builder = ServerBuilder::new(server_name);
    let bench_server = BenchService::new();
    add_service(bench_server.clone(), &mut builder).unwrap();
    let server = builder.build();
    net.add_server(server.clone());
    (net, server, bench_server)
}

fn bench_rpc(c: &mut Criterion) {
    let (net, server, _bench_server) = bench_suit();
    let server_name = server.name();
    let client_name = "client";
    let client = BenchClient::new(net.create_client(client_name.to_owned()));
    net.connect(client_name, server_name);
    net.enable(client_name, true);

    c.bench_function("rpc", |b| {
        b.iter(|| {
            black_box(block_on(async {
                client.handler(&BenchArgs { x: 111 }).await.unwrap()
            }));
        })
        // i7-8650U, 13 microseconds per RPC
    });
}

criterion_group!(benches, bench_rpc);
criterion_main!(benches);
