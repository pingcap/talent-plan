#[macro_use]
extern crate prost_derive;

use futures::Future;
use labrpc::*;

/// A Hand-written protobuf messages
#[derive(Clone, PartialEq, Message)]
pub struct Echo {
    #[prost(int64, tag = "1")]
    pub x: i64,
}

service! {
    service echo {
        rpc ping(Echo) returns (Echo);
    }
}
use echo::{add_service, Client, Service};

#[derive(Clone)]
struct EchoService;

impl Service for EchoService {
    fn ping(&self, input: Echo) -> RpcFuture<Echo> {
        Box::new(futures::future::result(Ok(input.clone())))
    }
}

fn main() {
    let rn = Network::new();
    let server_name = "echo_server";
    let mut builder = ServerBuilder::new(server_name.to_owned());
    add_service(EchoService, &mut builder).unwrap();
    let server = builder.build();
    rn.add_server(server.clone());

    let client_name = "client";
    let client = Client::new(rn.create_client(client_name.to_owned()));
    rn.enable(client_name, true);
    rn.connect(client_name, server_name);

    let reply = client.ping(&Echo { x: 777 }).wait().unwrap();
    assert_eq!(reply, Echo { x: 777 });
    println!("{:?}", reply);
}
