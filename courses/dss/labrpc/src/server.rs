use std::collections::hash_map::{Entry, HashMap};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use futures::future::{self, BoxFuture};

use crate::error::{Error, Result};

static ID_ALLOC: AtomicUsize = AtomicUsize::new(0);

pub type RpcFuture<T> = BoxFuture<'static, T>;

pub type Handler = dyn FnOnce(&[u8]) -> RpcFuture<Result<Vec<u8>>>;

pub trait HandlerFactory: Sync + Send + 'static {
    fn handler(&self, name: &'static str) -> Box<Handler>;
}

pub struct ServerBuilder {
    name: String,
    // Service name -> service methods
    pub(crate) services: HashMap<&'static str, Box<dyn HandlerFactory>>,
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
        factory: Box<dyn HandlerFactory>,
    ) -> Result<()> {
        match self.services.entry(service_name) {
            Entry::Occupied(_) => Err(Error::Other(format!(
                "{} has already registered",
                service_name
            ))),
            Entry::Vacant(entry) => {
                entry.insert(factory);
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

pub(crate) struct ServerCore {
    pub(crate) name: String,
    pub(crate) id: usize,

    pub(crate) services: HashMap<&'static str, Box<dyn HandlerFactory>>,
    pub(crate) count: AtomicUsize,
}

#[derive(Clone)]
pub struct Server {
    pub(crate) core: Arc<ServerCore>,
}

impl Server {
    pub fn count(&self) -> usize {
        self.core.count.load(Ordering::Relaxed)
    }

    pub fn name(&self) -> &str {
        &self.core.name
    }

    pub(crate) fn dispatch(&self, fq_name: &'static str, req: &[u8]) -> RpcFuture<Result<Vec<u8>>> {
        self.core.count.fetch_add(1, Ordering::Relaxed);
        let mut names = fq_name.split('.');
        let service_name = match names.next() {
            Some(n) => n,
            None => {
                return Box::pin(future::err(Error::Unimplemented(format!(
                    "unknown {}",
                    fq_name
                ))));
            }
        };
        let method_name = match names.next() {
            Some(n) => n,
            None => {
                return Box::pin(future::err(Error::Unimplemented(format!(
                    "unknown {}",
                    fq_name
                ))));
            }
        };
        if let Some(factory) = self.core.services.get(service_name) {
            let handle = factory.handler(method_name);
            handle(req)
        } else {
            Box::pin(future::err(Error::Unimplemented(format!(
                "unknown {}",
                fq_name
            ))))
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
