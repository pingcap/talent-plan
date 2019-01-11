extern crate bytes;
extern crate labcodec;
extern crate prost;

#[cfg(test)]
#[macro_use]
extern crate prost_derive;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

mod error;

pub use error::Error;

pub trait Service {
    fn name(&self) -> &'static str;
    fn dispatch(&self, method: &str, req: &[u8], rsp: &mut Vec<u8>) -> Result<(), Error>;
}

pub struct Server {
    services: HashMap<&'static str, Box<dyn Service>>,
    count: AtomicUsize,
}

impl Server {
    pub fn new() -> Server {
        Server {
            services: HashMap::new(),
            count: AtomicUsize::new(0),
        }
    }

    pub fn add_service(&mut self, svc: Box<dyn Service>) {
        self.services.insert(svc.name(), svc);
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    fn dispatch(&self, fq_name: &str, req: &[u8], rsp: &mut Vec<u8>) -> Result<(), Error> {
        self.count.fetch_add(1, Ordering::SeqCst);
        let mut parts = fq_name.split('.');
        let svc_name = parts.next().unwrap();
        let method_name = parts.next().unwrap();

        if let Some(svc) = self.services.get(&svc_name) {
            svc.dispatch(method_name, req, rsp)
        } else {
            Err(Error::Unimplemented(format!(
                "unknown service {} in {}",
                svc_name, fq_name
            )))
        }
    }
}

pub struct ClientEnd {
    endname: String, // this end-point's name
                     // ch      chan reqMsg   // copy of Network.endCh
                     // done    chan struct{} // closed when Network is cleaned up
}

pub struct Network {
    reliable: bool,
    // pause a long time on send on disabled connection
    long_delays: bool,
    // sometimes delay replies a long time
    long_reordering: bool,
    // ends, by name
    ends: HashMap<String, ClientEnd>,
    // by end name
    enabled: HashMap<String, bool>,
    // servers, by name
    servers: HashMap<String, Server>,
    // endname -> servername
    connections: HashMap<String, String>,
    count: AtomicUsize,
}

impl Network {
    pub fn new() -> Network {
        Network {
            reliable: false,
            long_delays: false,
            long_reordering: false,
            ends: HashMap::new(),
            enabled: HashMap::new(),
            servers: HashMap::new(),
            connections: HashMap::new(),
            count: AtomicUsize::new(0),
        }
    }

    fn cleanup(&mut self) {
        // unimplemented!()
    }
}

impl Drop for Network {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    use super::*;

    // Hand-written protobuf messages.
    #[derive(Clone, PartialEq, Message)]
    struct JunkArgs {
        #[prost(int64, tag = "1")]
        pub x: i64,
    }
    #[derive(Clone, PartialEq, Message)]
    struct JunkReply {
        #[prost(string, tag = "1")]
        pub x: String,
    }

    #[derive(Default)]
    struct JunkInner {
        log1: Vec<String>,
        log2: Vec<i64>,
    }
    struct JunkServer {
        inner: Mutex<JunkInner>,
    }
    impl JunkServer {
        fn new() -> JunkServer {
            JunkServer {
                inner: Mutex::new(JunkInner::default()),
            }
        }
    }
    impl JunkService for JunkServer {
        fn handler4(&self, args: JunkArgs) -> JunkReply {
            JunkReply {
                x: "pointer".to_owned(),
            }
        }
    }

    trait JunkService: Service {
        // We only supports protobuf messages.
        fn handler4(&self, args: JunkArgs) -> JunkReply;
    }
    impl<T: ?Sized + JunkService> Service for T {
        fn name(&self) -> &'static str {
            "junk"
        }
        fn dispatch(&self, method_name: &str, req: &[u8], rsp: &mut Vec<u8>) -> Result<(), Error> {
            match method_name {
                "handler4" => {
                    let request = labcodec::decode(req).map_err(Error::Decode)?;
                    let response = self.handler4(request);
                    labcodec::encode(&response, rsp).map_err(Error::Encode)
                }
                other => Err(Error::Unimplemented(format!(
                    "unknown method {} in {}",
                    other,
                    self.name()
                ))),
            }
        }
    }

    #[test]
    fn test_service_dispatch() {
        let rn = Network::new();
        let junk_server = JunkServer::new();
        let svc: Box<dyn Service> = Box::new(junk_server);
        let mut server = Server::new();
        server.add_service(svc);

        let mut buf = Vec::new();
        server.dispatch("junk.handler4", &[], &mut buf).unwrap();
        let rsp = labcodec::decode(&buf).unwrap();
        assert_eq!(
            JunkReply {
                x: "pointer".to_owned(),
            },
            rsp,
        );

        buf.clear();
        server
            .dispatch("junk.handler4", b"bad message", &mut buf)
            .unwrap_err();
        assert!(buf.is_empty());

        buf.clear();
        server
            .dispatch("badjunk.handler4", &[], &mut buf)
            .unwrap_err();
        assert!(buf.is_empty());

        buf.clear();
        server
            .dispatch("junk.badhandler", &[], &mut buf)
            .unwrap_err();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_basic() {
        // e := rn.MakeEnd("end1-99")

        // js := &JunkServer{}
        // svc := MakeService(js)

        // rs := MakeServer()
        // rs.AddService(svc)
        // rn.AddServer("server99", rs)

        // rn.Connect("end1-99", "server99")
        // rn.Enable("end1-99", true)

        // {
        // 	reply := ""
        // 	e.Call("JunkServer.Handler2", 111, &reply)
        // 	if reply != "handler2-111" {
        // 		t.Fatalf("wrong reply from Handler2")
        // 	}
        // }

        // {
        // 	reply := 0
        // 	e.Call("JunkServer.Handler1", "9099", &reply)
        // 	if reply != 9099 {
        // 		t.Fatalf("wrong reply from Handler1")
        // 	}
        // }
    }
}
