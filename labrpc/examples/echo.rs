#[macro_use]
extern crate prost_derive;
extern crate labcodec;
extern crate labrpc;

use labrpc::*;

/// A Hand-written protobuf messages
#[derive(Clone, PartialEq, Message)]
pub struct Echo {
    #[prost(int64, tag = "1")]
    pub x: i64,
}

service! {
    service echo {
        rpc ping(Echo) returns Echo;
    }
}
use echo::{add_service, Client, Service};

#[derive(Clone)]
struct EchoService;

impl Service for EchoService {
    fn ping(&self, input: Echo) -> Echo {
        input.clone()
    }
}

fn main() {
    let rn = Network::new();
    let server_name = "echo_server".to_owned();
    let mut builder = ServerBuilder::new(server_name.clone());
    let service = EchoService;
    add_service(&service, &mut builder).unwrap();
    let server = builder.build();
    rn.add_server(server.clone());

    let client_name = "client".to_owned();
    let client = Client::new(rn.create_end(client_name.clone()));
    rn.enable(client_name.clone(), true);
    rn.connect(client_name.clone(), server_name.clone());

    let reply = client.ping(&Echo { x: 777 }).unwrap();
    assert_eq!(reply, Echo { x: 777 });
    println!("{:?}", reply);
}
