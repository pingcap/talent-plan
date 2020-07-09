use futures::executor::block_on;
use prost_derive::Message;

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

#[async_trait::async_trait]
impl Service for EchoService {
    async fn ping(&self, input: Echo) -> Result<Echo> {
        Ok(input)
    }
}

fn main() {
    let rn = Network::new();
    let server_name = "echo_server";
    let mut builder = ServerBuilder::new(server_name.to_owned());
    add_service(EchoService, &mut builder).unwrap();
    let server = builder.build();
    rn.add_server(server);

    let client_name = "client";
    let client = Client::new(rn.create_client(client_name.to_owned()));
    rn.enable(client_name, true);
    rn.connect(client_name, server_name);

    let reply = block_on(async { client.ping(&Echo { x: 777 }).await.unwrap() });
    assert_eq!(reply, Echo { x: 777 });
    println!("{:?}", reply);
}
