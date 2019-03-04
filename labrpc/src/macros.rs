#[macro_export]
macro_rules! service {
    () => {
        compile_error!("empty service is not allowed");
    };
    (
        $(#[$service_attr:meta])*
        service $svc_name:ident {
            $(
                $(#[$method_attr:meta])*
                rpc $method_name:ident($input:ty) returns ($output:ty);
            )*
        }
    ) => {
        $(#[$service_attr])*
        pub mod $svc_name {
            // In order to find input and output.
            use super::*;
            // $( use super::$input; )*
            // $( use super::$output;)*

            extern crate futures as __futures;

            pub trait Service: Clone + Send + 'static {
                $(
                    $(#[$method_attr])*
                    fn $method_name(&self, $input) -> $output;
                )*
            }

            #[derive(Clone)]
            pub struct Client {
                client: $crate::Client,
            }
            impl Client {
                pub fn new(client: $crate::Client) -> Client {
                    Client { client }
                }

                pub fn spawn<F>(&self, f: F)
                where F: __futures::Future<Item=(), Error=()> + Send + 'static
                {
                    self.client.worker.spawn(f).forget()
                }

                $(pub fn $method_name(&self, args: &$input) -> Box<dyn __futures::Future<Item = $output, Error = $crate::Error> + Send + 'static> {
                    let fq_name = concat!(stringify!($svc_name), ".", stringify!($method_name));
                    self.client.call(fq_name, args)
                })*
            }

            pub fn add_service<T: Service>(svc: T, builder: &mut $crate::ServerBuilder) -> $crate::Result<()> {
                use ::std::sync::Mutex;
                struct Factory<S> {
                    svc: Mutex<S>,
                }
                impl<S: Service> $crate::HandlerFactory for Factory<S> {
                    fn handler(&self, name: &'static str) -> Box<$crate::Handler> {
                        let s = self.svc.lock().unwrap().clone();
                        Box::new(move |req, rsp| {
                            match name {
                                $(stringify!($method_name) => {
                                    let request = labcodec::decode(req).map_err($crate::Error::Decode)?;
                                    let response = s.$method_name(request);
                                    labcodec::encode(&response, rsp).map_err($crate::Error::Encode)
                                })*
                                other => {
                                    Err($crate::Error::Unimplemented(
                                        format!("unknown {} in {}", other, stringify!($svc_name))
                                    ))
                                }
                            }
                        })
                    }
                }

                let fact = Factory {
                    svc: Mutex::new(svc),
                };

                builder.add_service(stringify!($svc_name), Box::new(fact))
            }
        }
    };
}
