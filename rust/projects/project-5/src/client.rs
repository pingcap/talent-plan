use crate::common::{GetResponse, RemoveResponse, Request, SetResponse};
use crate::{KvsError, Result};
use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};
use std::io::{BufReader, BufWriter, Write};
use std::net::SocketAddr;
use tokio::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio_serde_json::{ReadJson, WriteJson};

/// Key value store client
pub struct KvsClient {
    tcp: Option<TcpStream>,
}

impl KvsClient {
    /// Connect to `addr` to access `KvsServer`.
    pub fn connect(addr: SocketAddr) -> impl Future<Item = Self, Error = KvsError> {
        TcpStream::connect(&addr)
            .map(|tcp| KvsClient { tcp: Some(tcp) })
            .map_err(|e| e.into())
    }

    /// Get the value of a given key from the server.
    pub fn get(
        mut self,
        key: String,
    ) -> impl Future<Item = (Option<String>, Self), Error = KvsError> {
        let tcp = self.tcp.take().unwrap();
        let write_json = WriteJson::new(FramedWrite::new(tcp, LengthDelimitedCodec::new()));
        let tcp = write_json
            .send(Request::Get { key })
            .map(|serialized| serialized.into_inner().into_inner());
        tcp.and_then(|tcp| {
            let read_json = ReadJson::new(FramedRead::new(tcp, LengthDelimitedCodec::new()));
            read_json.into_future().map_err(|(err, _)| err)
        })
        .map_err(|e| e.into())
        .and_then(move |(resp, read_json)| {
            self.tcp = Some(read_json.into_inner().into_inner());
            match resp {
                Some(GetResponse::Ok(value)) => Ok((value, self)),
                Some(GetResponse::Err(msg)) => Err(KvsError::StringError(msg)),
                None => Err(KvsError::StringError("No response received".to_owned())),
            }
        })
    }

    /// Set the value of a string key in the server.
    pub fn set(mut self, key: String, value: String) -> impl Future<Item = Self, Error = KvsError> {
        let tcp = self.tcp.take().unwrap();
        let write_json = WriteJson::new(FramedWrite::new(tcp, LengthDelimitedCodec::new()));
        let tcp = write_json
            .send(Request::Set { key, value })
            .map(|serialized| serialized.into_inner().into_inner());
        tcp.and_then(|tcp| {
            let read_json = ReadJson::new(FramedRead::new(tcp, LengthDelimitedCodec::new()));
            read_json.into_future().map_err(|(err, _)| err)
        })
        .map_err(|e| e.into())
        .and_then(move |(resp, read_json)| {
            self.tcp = Some(read_json.into_inner().into_inner());
            match resp {
                Some(SetResponse::Ok(_)) => Ok(self),
                Some(SetResponse::Err(msg)) => Err(KvsError::StringError(msg)),
                None => Err(KvsError::StringError("No response received".to_owned())),
            }
        })
    }

    /// Remove a string key in the server.
    pub fn remove(mut self, key: String) -> impl Future<Item = Self, Error = KvsError> {
        let tcp = self.tcp.take().unwrap();
        let write_json = WriteJson::new(FramedWrite::new(tcp, LengthDelimitedCodec::new()));
        let tcp = write_json
            .send(Request::Remove { key })
            .map(|serialized| serialized.into_inner().into_inner());
        tcp.and_then(|tcp| {
            let read_json = ReadJson::new(FramedRead::new(tcp, LengthDelimitedCodec::new()));
            read_json.into_future().map_err(|(err, _)| err)
        })
        .map_err(|e| e.into())
        .and_then(move |(resp, read_json)| {
            self.tcp = Some(read_json.into_inner().into_inner());
            match resp {
                Some(RemoveResponse::Ok(_)) => Ok(self),
                Some(RemoveResponse::Err(msg)) => Err(KvsError::StringError(msg)),
                None => Err(KvsError::StringError("No response received".to_owned())),
            }
        })
    }
}
