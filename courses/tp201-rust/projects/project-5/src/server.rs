use crate::common::{Request, Response};
use crate::{KvsEngine, KvsError, Result};
use std::net::SocketAddr;
use tokio::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
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
    let resp_stream = read_json
        .map_err(KvsError::from)
        .and_then(
            move |req| -> Box<dyn Future<Item = Response, Error = KvsError> + Send> {
                match req {
                    Request::Get { key } => Box::new(engine.get(key).map(Response::Get)),
                    Request::Set { key, value } => {
                        Box::new(engine.set(key, value).map(|_| Response::Set))
                    }
                    Request::Remove { key } => {
                        Box::new(engine.remove(key).map(|_| Response::Remove))
                    }
                }
            },
        )
        .then(|resp| -> Result<Response> {
            match resp {
                Ok(resp) => Ok(resp),
                Err(e) => Ok(Response::Err(format!("{}", e))),
            }
        });
    let write_json = WriteJson::new(FramedWrite::new(write_half, LengthDelimitedCodec::new()));
    write_json
        .sink_map_err(KvsError::from)
        .send_all(resp_stream)
        .map(|_| ())
}
