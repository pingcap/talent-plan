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

            #[async_trait::async_trait]
            pub trait Service: Clone + Send + 'static {
                $(
                    $(#[$method_attr])*
                    async fn $method_name(&self, req: $input) -> $crate::Result<$output>;
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
                where F: __futures::Future<Output = ()> + Send + 'static
                {
                    self.client.worker.spawn_ok(f);
                }

                $(pub fn $method_name(&self, args: &$input) -> $crate::RpcFuture<$crate::Result<$output>> {
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
                        Box::new(move |req| {
                            match name {
                                $(stringify!($method_name) => {
                                    let request = match labcodec::decode(req) {
                                        Ok(req) => req,
                                        Err(e) => return Box::pin(__futures::future::err(
                                            $crate::Error::Decode(e)
                                        )),
                                    };
                                    Box::pin(async move {
                                        let f = s.$method_name(request);
                                        let resp = f.await;
                                        match resp {
                                            Ok(resp) => {
                                                let mut rsp = vec![];
                                                labcodec::encode(&resp, &mut rsp).map_err($crate::Error::Encode)?;
                                                Ok(rsp)
                                            }
                                            Err(e) => Err(e),
                                        }
                                    })
                                })*
                                other => {
                                    Box::pin(__futures::future::err(
                                        $crate::Error::Unimplemented(
                                            format!("unknown {} in {}", other, stringify!($svc_name))
                                        )
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
