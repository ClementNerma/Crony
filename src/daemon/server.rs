use crate::service;

service!(
    daemon {
        fn hello(__: ()) -> Result<String> {
            Ok("Hello".to_string())
        }
    }
);
