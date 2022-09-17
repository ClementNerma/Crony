use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[macro_export]
macro_rules! service {
    ($service_name:ident { $(fn $fn_name:ident($fn_arg_name:ident: $fn_arg_type:tt) -> Result<$fn_ret_type:tt> $content:block)+ }) => {
        pub mod $service_name {
            pub trait Server {
                fn process_unchecked(&self, req: RequestContent) -> ::anyhow::Result<ResponseContent>;
            }

            pub trait Client {
                fn send_unchecked(&self, req: RequestContent) -> ::anyhow::Result<ResponseContent>;
            }

            pub type Request = $crate::ipc::Request<RequestContent>;
            pub type Response = $crate::ipc::Request<ResponseContent>;

            #[derive(::serde::Serialize, ::serde::Deserialize)]
            #[allow(non_camel_case_types)]
            pub enum RequestContent {
                $($fn_name { $fn_arg_name: $fn_arg_type }),+
            }

            #[derive(::serde::Serialize, ::serde::Deserialize)]
            #[allow(non_camel_case_types)]
            pub enum ResponseContent {
                $($fn_name($fn_ret_type)),+
            }

            mod handlers {
                $(pub(super) fn $fn_name($fn_arg_name: $fn_arg_type) -> ::anyhow::Result<$fn_ret_type> $content)+
            }

            pub mod senders {
                $(pub fn $fn_name(client: impl super::Client, $fn_arg_name: $fn_arg_type) -> ::anyhow::Result<$fn_ret_type> {
                    match super::Client::send_unchecked(&client, super::RequestContent::$fn_name { $fn_arg_name })? {
                        super::ResponseContent::$fn_name(output) => Ok(output),

                        #[allow(unreachable_patterns)]
                        _ => ::anyhow::bail!("Invalid unchecked response variant returned by service client")
                    }
                })+
            }

            pub fn process(req: RequestContent) -> $crate::ipc::Processed<ResponseContent> {
                match req {
                    $(RequestContent::$fn_name { $fn_arg_name } => handlers::$fn_name($fn_arg_name).map(|value| ResponseContent::$fn_name(value)).map_err(|error| format!("{:?}", error))),+
                }
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
    pub result: ::std::result::Result<T, String>,
}

pub type Processed<T> = ::std::result::Result<T, String>;
