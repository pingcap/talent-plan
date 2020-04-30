use std::fmt;
use std::sync::{Arc, Mutex};

use futures::channel::mpsc::UnboundedSender;
use futures::channel::oneshot;
use futures::executor::ThreadPool;
use futures::future::{self, FutureExt};

use crate::error::{Error, Result};
use crate::server::RpcFuture;

pub struct Rpc {
    pub(crate) client_name: String,
    pub(crate) fq_name: &'static str,
    pub(crate) req: Option<Vec<u8>>,
    pub(crate) resp: Option<oneshot::Sender<Result<Vec<u8>>>>,
    pub(crate) hooks: Arc<Mutex<Option<Arc<dyn RpcHooks>>>>,
}

impl Rpc {
    pub(crate) fn take_resp_sender(&mut self) -> Option<oneshot::Sender<Result<Vec<u8>>>> {
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
    pub(crate) name: String,
    // copy of Network.sender
    pub(crate) sender: UnboundedSender<Rpc>,
    pub(crate) hooks: Arc<Mutex<Option<Arc<dyn RpcHooks>>>>,

    pub worker: ThreadPool,
}

impl Client {
    pub fn call<Req, Rsp>(&self, fq_name: &'static str, req: &Req) -> RpcFuture<Result<Rsp>>
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

        Box::pin(rx.then(|res| async move {
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
