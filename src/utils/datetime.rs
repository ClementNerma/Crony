use anyhow::Context;
use once_cell::sync::Lazy;
use time::{format_description, Duration, OffsetDateTime, UtcOffset};

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

pub fn human_datetime(datetime: OffsetDateTime) -> String {
    datetime
        .format(&format_description::well_known::Rfc2822)
        .context("Failed to stringify start date")
        .unwrap()
}

pub fn human_duration(duration: Duration) -> String {
    let mut secs = duration.whole_seconds();

    assert!(
        secs >= 0,
        "Number of seconds is negative in provided duration"
    );

    let mut times = String::new();

    if secs > 86400 {
        times.push_str(&format!("{}D", secs / 86400));
        secs = secs - (secs / 86400 * 86400);
    }

    if secs > 3600 {
        times.push_str(&format!("{}h", secs / 3600));
        secs = secs - (secs / 3600 * 3600);
    }

    if secs > 60 {
        times.push_str(&format!("{}m", secs / 60));
        secs = secs - (secs / 60 * 60);
    }

    times.push_str(&format!("{secs}s"));
    times
}
