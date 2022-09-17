use anyhow::Context;
use time::{format_description, OffsetDateTime};

pub fn get_now() -> OffsetDateTime {
    // OffsetDateTime::now_local()
    //     .context("Failed to determine current date/time")
    //     .unwrap()

    OffsetDateTime::now_utc()
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
