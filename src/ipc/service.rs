use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! service {
    ($service_name:ident ($mod:ident) {
        $(fn $fn_name:ident($($fn_arg_name:ident: $fn_arg_type:ty)?)$( -> $fn_ret_type:ty)?;)+
    }) => {
        pub mod $service_name {
            use ::std::sync::Arc;

            use ::serde::{Serialize, Deserialize};
            use ::anyhow::{bail, Result};

            use $crate::ipc::SocketClient;

            use super::$mod::{self as functions, State};

            #[derive(Serialize, Deserialize)]
            #[allow(non_camel_case_types)]
            pub enum RequestContent {
                $($fn_name $({ $fn_arg_name: $fn_arg_type })?),+
            }

            #[derive(Serialize, Deserialize)]
            #[allow(non_camel_case_types)]
            pub enum ResponseContent {
                $($fn_name($crate::service!(@ty $($fn_ret_type)?))),+
            }

            mod handlers {
                $(pub(super) fn $fn_name(#[allow(unused_variables)] state: super::Arc<super::State>$(, $fn_arg_name: $fn_arg_type)?)$( -> $fn_ret_type)? {
                    super::functions::$fn_name(state$(, $fn_arg_name)?)
                })+
            }

            pub fn process(req: RequestContent, state: Arc<State>) -> ResponseContent {
                match req {
                    $(RequestContent::$fn_name $({ $fn_arg_name })? => ResponseContent::$fn_name(handlers::$fn_name(state, $($fn_arg_name)?))),+
                }
            }

            pub struct Client {
                pub inner: SocketClient<RequestContent, ResponseContent>
            }

            impl Client {
                $(pub fn $fn_name(&mut self$(, $fn_arg_name: $fn_arg_type)?) -> Result<$crate::service!(@ty $($fn_ret_type)?)> {
                    match self.inner.send_unchecked(RequestContent::$fn_name $({ $fn_arg_name })?)? {
                        ResponseContent::$fn_name ($crate::service!(@pat $($fn_ret_type)? => output)) => Ok($crate::service!(@expr $($fn_ret_type)? => output)),

                        #[allow(unreachable_patterns)]
                        _ => bail!("Invalid unchecked response variant returned by service client"),
                    }
                })+
            }
        }
    };

    (@ty $type:ty) => { $type };
    (@ty) => { () };

    (@pat $type:ty => $pat:pat) => { $pat };
    (@pat => $pat:pat) => { () };

    (@expr $type:ty => $expr:expr) => { $expr };
    (@expr => $expr:expr) => { () };
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
