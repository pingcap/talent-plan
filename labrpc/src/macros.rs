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
                rpc $method_name:ident( $input:ident ) returns $output:ident;
            )*
        }
    ) => {
        $(#[$service_attr])*
        pub mod $svc_name {
            // In order to find input and output.
            use super::*;
            // $( use super::$input; )*
            // $( use super::$output;)*

            pub trait Service: Clone + Send + Sync + 'static {
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
                $(
                    pub fn $method_name(&self, args: &$input) -> $crate::Result<$output> {
                        let fq_name = concat!(stringify!($svc_name), ".", stringify!($method_name));
                        self.client.call(fq_name, args)
                    }
                )*
            }

            pub fn add_service<T: Service>(svc: &T, builder: &mut $crate::ServerBuilder) -> $crate::Result<()> {
                $({
                    let fq_name = concat!(stringify!($svc_name), ".", stringify!($method_name));
                    let s = svc.clone();
                    builder.add_handler(fq_name, Box::new(move |req, rsp| {
                        let request = labcodec::decode(req).map_err($crate::Error::Decode)?;
                        let response = s.$method_name(request);
                        labcodec::encode(&response, rsp).map_err($crate::Error::Encode)
                    }))?;
                })*

                Ok(())
            }
        }
    };
}
