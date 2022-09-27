use once_cell::sync::Lazy;
use time::{OffsetDateTime, UtcOffset};

use crate::warn;

// Required as the offset can fail to be get in some contexts
static OFFSET: Lazy<UtcOffset> = Lazy::new(|| {
    UtcOffset::local_offset_at(OffsetDateTime::now_utc()).unwrap_or_else(|_| {
        warn!("Failed to determine local offset, UTC will be used instead");
        UtcOffset::UTC
    })
});

pub fn get_now() -> OffsetDateTime {
    // OffsetDateTime::now_local()
    //     .context("Failed to determine current date/time")
    //     .unwrap()

    OffsetDateTime::now_utc().to_offset(*OFFSET)
}

pub fn get_now_second_precision() -> OffsetDateTime {
    second_precision(get_now())
}

pub fn second_precision(moment: OffsetDateTime) -> OffsetDateTime {
    moment.replace_nanosecond(0).unwrap()
}
