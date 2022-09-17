use crate::service;

service!(
    daemon {
        fn greet(name: String) -> Result<String> {
            Ok(format!("Hello, '{name}'!"))
        }
    }
);
