use anyhow::Result;
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! service {
    ($service_name:ident ($state_type:ty) { $(fn $fn_name:ident($state_arg_name:ident, $fn_arg_name:ident: $fn_arg_type:ty) -> Result<$fn_ret_type:ty> $content:block)+ }) => {
        type ___State = $state_type;

        pub mod $service_name {
            use ::std::sync::Arc;
            use ::anyhow::Result;
            use ::serde::{Serialize, Deserialize};
            use $crate::ipc::{ServiceClient, Processed};
            use super::___State as State;

            #[derive(Serialize, Deserialize)]
            #[allow(non_camel_case_types)]
            pub enum RequestContent {
                $($fn_name { $fn_arg_name: $fn_arg_type }),+
            }

            #[derive(Serialize, Deserialize)]
            #[allow(non_camel_case_types)]
            pub enum ResponseContent {
                $($fn_name($fn_ret_type)),+
            }

            mod handlers {
                $(pub(super) fn $fn_name(#[allow(unused_variables)] $state_arg_name: super::Arc<super::State>, $fn_arg_name: $fn_arg_type) -> super::Result<$fn_ret_type> $content)+
            }

            pub mod senders {
                use ::anyhow::bail;
                use super::{ServiceClient, RequestContent, ResponseContent, Processed};

                $(pub fn $fn_name(client: &mut impl ServiceClient<RequestContent, ResponseContent>, $fn_arg_name: $fn_arg_type) -> Processed<$fn_ret_type> {
                    match client.send_unchecked(RequestContent::$fn_name { $fn_arg_name })? {
                        Ok(ResponseContent::$fn_name(output)) => Ok(Ok(output)),

                        #[allow(unreachable_patterns)]
                        Ok(_) => bail!("Invalid unchecked response variant returned by service client"),

                        Err(err) => Ok(Err(err))
                    }
                })+
            }

            pub fn process(req: RequestContent, state: Arc<State>) -> Result<ResponseContent, String> {
                match req {
                    $(RequestContent::$fn_name { $fn_arg_name } => handlers::$fn_name(state, $fn_arg_name).map(|value| ResponseContent::$fn_name(value)).map_err(|error| format!("{:?}", error))),+
                }
            }

            pub trait Client {
                type Client: ServiceClient<RequestContent, ResponseContent>;

                fn retrieve_client(&mut self) -> &mut Self::Client;

                $(fn $fn_name(&mut self, $fn_arg_name: $fn_arg_type) -> Processed<$fn_ret_type> {
                    senders::$fn_name(self.retrieve_client(), $fn_arg_name)
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
    pub result: Result<T, String>,
}

pub type Processed<T> = Result<Result<T, String>>;

// pub trait Server<Req, Res> {
//     fn process_unchecked(&self, req: Req) -> Result<Res>;
// }

pub trait ServiceClient<Req, Res> {
    fn send_unchecked(&mut self, req: Req) -> Processed<Res>;
}
