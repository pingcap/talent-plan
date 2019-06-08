use crate::common::{Request, Response};
use crate::thread_pool::ThreadPool;
use crate::{KvsEngine, KvsError, Result};
use serde_json::Deserializer;
use std::io::{BufReader, BufWriter, Write};
use std::net::SocketAddr;
use tokio::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio_serde_json::{ReadJson, WriteJson};

/// The server of a key value store.
pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine> KvsServer<E> {
    /// Create a `KvsServer` with a given storage engine.
    pub fn new(engine: E) -> Self {
        KvsServer { engine }
    }

    /// Run the server listening on the given address
    pub fn run(self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(&addr)?;
        let server = listener
            .incoming()
            .map_err(|e| error!("IO error: {}", e))
            .for_each(move |tcp| {
                let engine = self.engine.clone();
                serve(engine, tcp).map_err(|e| error!("Error on serving client: {}", e))
            });
        tokio::run(server);
        Ok(())
    }
}

fn serve<E: KvsEngine>(engine: E, tcp: TcpStream) -> impl Future<Item = (), Error = KvsError> {
    let (read_half, write_half) = tcp.split();
    let read_json = ReadJson::new(FramedRead::new(read_half, LengthDelimitedCodec::new()));
    let write_json = WriteJson::new(FramedWrite::new(write_half, LengthDelimitedCodec::new()));
    write_json
        .send_all(read_json.map(move |req| match req {
            Request::Get { key } => match engine.get(key) {
                Ok(value) => Response::Get(value),
                Err(e) => Response::Err(format!("{}", e)),
            },
            Request::Set { key, value } => match engine.set(key, value) {
                Ok(_) => Response::Set,
                Err(e) => Response::Err(format!("{}", e)),
            },
            Request::Remove { key } => match engine.remove(key) {
                Ok(_) => Response::Remove,
                Err(e) => Response::Err(format!("{}", e)),
            },
        }))
        .map(|_| ())
        .map_err(|e| e.into())
}
