use std::{convert::TryFrom, ops::Add};

use anyhow::Result;
use time::{Duration, Month, OffsetDateTime};

use crate::at::{At, Occurrences};

// NOTE: *VERY IMPORTANT* (TODO)
// This occurrence finder engine only works with dates that keep validity
// when changed ; which means that using a day like 30 will make it return an Err()
// if we're in february.
pub fn get_upcoming_moment(after: OffsetDateTime, at: &At) -> Result<OffsetDateTime> {
    let next = after;

    let next = match &at.seconds {
        Occurrences::First => {
            if next.second() == 0 {
                next
            } else {
                next.replace_second(0).unwrap().add(Duration::minutes(1))
            }
        }
        Occurrences::Every => next, //.add(Duration::seconds(1)),
        Occurrences::Once(at) => {
            if *at > next.second() {
                next.replace_second(*at).unwrap()
            } else {
                next.replace_second(*at).unwrap().add(Duration::minutes(1))
            }
        }
        Occurrences::Multiple(at) => {
            let (nearest, overflow) = nearest_value(at, next.second(), 60);
            next.replace_second(nearest)
                .unwrap()
                .add(Duration::minutes(if overflow { 1 } else { 0 }))
        }
    };

    let next = match &at.minutes {
        Occurrences::First => {
            if next.minute() == 0 {
                next
            } else {
                next.replace_minute(0).unwrap().add(Duration::hours(1))
            }
        }
        Occurrences::Every => next, //.add(Duration::hours(1)),
        Occurrences::Once(at) => {
            if *at > next.minute() {
                next.replace_minute(*at).unwrap()
            } else {
                next.replace_minute(*at).unwrap().add(Duration::hours(1))
            }
        }
        Occurrences::Multiple(at) => {
            let (nearest, overflow) = nearest_value(at, next.minute(), 60);
            next.replace_minute(nearest)
                .unwrap()
                .add(Duration::hours(if overflow { 1 } else { 0 }))
        }
    };

    let next = match &at.hours {
        Occurrences::First => {
            if next.hour() == 0 {
                next
            } else {
                next.replace_hour(0).unwrap().add(Duration::days(1))
            }
        }
        Occurrences::Every => next, //.add(Duration::days(1)),
        Occurrences::Once(at) => {
            if *at > next.hour() {
                next.replace_hour(*at).unwrap()
            } else {
                next.replace_hour(*at).unwrap().add(Duration::days(1))
            }
        }
        Occurrences::Multiple(at) => {
            let (nearest, overflow) = nearest_value(at, next.hour(), days_in_current_month(next));
            next.replace_hour(nearest)
                .unwrap()
                .add(Duration::days(if overflow { 1 } else { 0 }))
        }
    };

    let next = match &at.days {
        Occurrences::First => {
            if next.day() == 1 {
                next
            } else {
                next_month(next.replace_day(1).unwrap())
            }
        }
        Occurrences::Every => next, //.add(Duration::days(1)),
        Occurrences::Once(at) => {
            if *at > next.day() {
                next.replace_day(*at).unwrap()
            } else {
                next_month(next).replace_day(*at)?
            }
        }
        Occurrences::Multiple(at) => {
            let (nearest, overflow) = nearest_value(at, next.day(), days_in_current_month(next));

            let mut next = next.replace_day(nearest)?;

            if overflow {
                next = next_month(next);
            }

            next
        }
    };

    let next = match &at.months {
        Occurrences::First => {
            if next.month() == Month::January {
                next
            } else {
                next_year(next).replace_month(Month::January).unwrap()
            }
        }
        Occurrences::Every => next,
        Occurrences::Once(at) => {
            if *at > next.month().into() {
                next.replace_month(Month::try_from(*at).unwrap())?
            } else {
                next_year(next).replace_month(Month::try_from(*at).unwrap())?
            }
        }
        Occurrences::Multiple(at) => {
            let (nearest, overflow) = nearest_value(at, next.month().into(), 12);

            let mut next = next.replace_month(Month::try_from(nearest).unwrap())?;

            if overflow {
                next = next_year(next);
            }

            next
        }
    };

    Ok(next)
}

fn nearest_value(candidates: &[u8], curr: u8, total: u8) -> (u8, bool) {
    assert!(
        !candidates.is_empty(),
        "Candidates slice for the nearest value is empty!"
    );

    let mut nearest = (std::u8::MAX, std::u8::MAX, false);

    for candidate in candidates {
        // Required
        if *candidate == curr {
            continue;
        }

        let overflow = *candidate < curr;

        let distance = if overflow {
            *candidate + total - curr
        } else {
            *candidate - curr
        };

        if distance < nearest.0 {
            nearest = (distance, *candidate, overflow);
        }
    }

    (nearest.1, nearest.2)
}

fn days_in_current_month(from: OffsetDateTime) -> u8 {
    let mut date = from;
    let start_month = date.month();

    let mut days = date.day();

    while date.month() == start_month {
        date = date.add(Duration::days(1));
        days += 1;
    }

    days - 1
}

fn next_month(from: OffsetDateTime) -> OffsetDateTime {
    if from.month() == Month::December {
        next_year(from).replace_month(Month::January).unwrap()
    } else {
        from.add(Duration::days(days_in_current_month(from).into()))
    }
}

fn next_year(from: OffsetDateTime) -> OffsetDateTime {
    from.replace_year(from.year() + 1).unwrap()
}
