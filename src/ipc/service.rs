use anyhow::Result;
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! service {
    ($service_name:ident ($mod:ident) {
        $(fn $fn_name:ident($($fn_arg_name:ident: $fn_arg_type:ty)?) -> $fn_ret_type:ty;)+
    }) => {
        pub mod $service_name {
            use ::std::sync::Arc;

            use ::serde::{Serialize, Deserialize};
            use ::anyhow::Result;

            use $crate::ipc::{ServiceClient};
            use super::$mod::{self as functions, State};

            #[derive(Serialize, Deserialize)]
            #[allow(non_camel_case_types)]
            pub enum RequestContent {
                $($fn_name $({ $fn_arg_name: $fn_arg_type })?),+
            }

            #[derive(Serialize, Deserialize)]
            #[allow(non_camel_case_types)]
            pub enum ResponseContent {
                $($fn_name($fn_ret_type)),+
            }

            mod handlers {
                $(pub(super) fn $fn_name(#[allow(unused_variables)] state: super::Arc<super::State>$(, $fn_arg_name: $fn_arg_type)?) -> $fn_ret_type {
                    super::super::$mod::$fn_name(state$(, $fn_arg_name)?)
                })+
            }

            pub mod senders {
                use ::anyhow::{bail, Result};
                use super::{ServiceClient, RequestContent, ResponseContent};

                $(pub fn $fn_name(client: &mut impl ServiceClient<RequestContent, ResponseContent>$(, $fn_arg_name: $fn_arg_type)?) -> Result<$fn_ret_type> {
                    match client.send_unchecked(RequestContent::$fn_name $({ $fn_arg_name })?)? {
                        ResponseContent::$fn_name(output) => Ok(output),

                        #[allow(unreachable_patterns)]
                        _ => bail!("Invalid unchecked response variant returned by service client"),
                    }
                })+
            }

            pub fn process(req: RequestContent, state: Arc<State>) -> ResponseContent {
                match req {
                    $(RequestContent::$fn_name $({ $fn_arg_name })? => ResponseContent::$fn_name(handlers::$fn_name(state, $($fn_arg_name)?))),+
                }
            }

            pub trait Client {
                type Client: ServiceClient<RequestContent, ResponseContent>;

                fn retrieve_client(&mut self) -> &mut Self::Client;

                $(fn $fn_name(&mut self $(, $fn_arg_name: $fn_arg_type)?) -> Result<$fn_ret_type> {
                    senders::$fn_name(self.retrieve_client() $(, $fn_arg_name)?)
                })+
            }
        }
    };
}

#[derive(Serialize, Deserialize)]
pub struct Request<T> {
    pub id: u64,
    pub content: T,
}

#[derive(Serialize, Deserialize)]
pub struct Response<T> {
    pub for_id: u64,
    pub result: T,
}

pub trait ServiceClient<Req, Res> {
    fn send_unchecked(&mut self, req: Req) -> Result<Res>;
}
