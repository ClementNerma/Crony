use std::{thread, time::Duration};

pub fn sleep_ms(millis: u64) {
    thread::sleep(Duration::from_millis(millis))
}
