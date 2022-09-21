use std::{convert::TryFrom, ops::Add};

use anyhow::{Context, Result};
use time::{Duration, Month, OffsetDateTime};

use crate::{
    at::{At, Occurrences},
    datetime::second_precision,
};

// NOTE: This function will fail to run when providing an invalid 'at'
//  e.g. day = 30 ; month = 2
pub fn get_upcoming_moment(after: OffsetDateTime, at: &At) -> Result<OffsetDateTime> {
    let next = after;
    let global_at = at;

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

    let set_seconds = next.second();

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

    // Required check for leap seconds
    if next.second() != set_seconds {
        return get_upcoming_moment(next, at);
    }

    let set_minutes = next.minute();

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

    if next.second() != set_seconds || next.minute() != set_minutes {
        return get_upcoming_moment(next, at);
    }

    let set_hours = next.hour();

    let next = match &at.days {
        Occurrences::First => {
            if next.day() == 1 {
                next
            } else {
                next_month(next.replace_day(1).unwrap())
            }
        }
        Occurrences::Every => next, //.add(Duration::days(1)),
        Occurrences::Once(at) => month_with_day(next, *at, *at <= next.day()),
        Occurrences::Multiple(at) => {
            let (nearest, overflow) = nearest_value(at, next.day(), days_in_current_month(next));

            let mut next = month_with_day(next, nearest, true);

            if overflow {
                next = next_month(next);
            }

            next
        }
    };

    if next.second() != set_seconds || next.minute() != set_minutes || next.hour() != set_hours {
        return get_upcoming_moment(next, at);
    }

    let set_day = next.day();

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
            let goal = Month::try_from(*at).unwrap();

            if *at > next.month().into() {
                let mut next = next;

                'result: loop {
                    for _ in 1..12 {
                        if next.month() == goal {
                            break 'result next;
                        }

                        next = next_month(next);
                    }
                }
            } else {
                let mut found = None;

                for years in 0..4 {
                    if let Ok(next) = next
                        .replace_year(next.year() + years + 1)
                        .and_then(|date| date.replace_month(goal))
                    {
                        found = Some(next);
                        break;
                    }
                }

                found.with_context(|| {
                    format!("Failed to determine a valid month/year couple for: {global_at}")
                })?
            }
        }
        Occurrences::Multiple(at) => {
            let nearest = nearest_values(at, next.month().into(), 12);

            let mut nearest = nearest
                .into_iter()
                .map(|(_, month, overflow)| {
                    (0..4)
                        .filter_map(|years| {
                            next.replace_year(next.year() + years + if overflow { 1 } else { 0 })
                                .and_then(|next| {
                                    next.replace_month(Month::try_from(month).unwrap())
                                })
                                .ok()
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .next()
                        .with_context(|| {
                            format!("Failed to determine a valid date for: {global_at}")
                        })
                })
                .collect::<Result<Vec<_>>>()?;

            nearest.sort_by_key(|moment| *moment - after);

            *nearest.first().unwrap()
        }
    };

    if next.second() != set_seconds
        || next.minute() != set_minutes
        || next.hour() != set_hours
        || next.day() != set_day
    {
        return get_upcoming_moment(next, at);
    }

    Ok(second_precision(next))
}

pub fn get_new_upcoming_moment(
    after: OffsetDateTime,
    at: &At,
    last: OffsetDateTime,
) -> Result<OffsetDateTime> {
    let upcoming = get_upcoming_moment(after, at)?;

    if upcoming != last {
        Ok(upcoming)
    } else {
        get_upcoming_moment(after.add(Duration::seconds(1)), at)
    }
}

fn nearest_value(candidates: &[u8], curr: u8, total: u8) -> (u8, bool) {
    assert!(
        !candidates.is_empty(),
        "Candidates slice for the nearest value is empty!"
    );

    let (_, nearest, overflow) = candidates
        .iter()
        .map(|candidate| {
            let (distance, overflow) = distance_from(*candidate, curr, total);
            (distance, *candidate, overflow)
        })
        .min_by_key(|(distance, _, _)| *distance)
        .unwrap();

    (nearest, overflow)
}

fn nearest_values(candidates: &[u8], curr: u8, total: u8) -> Vec<(u8, u8, bool)> {
    let mut nearest = candidates
        .iter()
        .map(|candidate| {
            let (distance, overflow) = distance_from(*candidate, curr, total);
            (distance, *candidate, overflow)
        })
        .collect::<Vec<_>>();

    nearest.sort_by_key(|(distance, _, _)| *distance);
    nearest
}

fn distance_from(candidate: u8, curr: u8, total: u8) -> (u8, bool) {
    let overflow = candidate < curr;

    let distance = if overflow {
        candidate + total - curr
    } else {
        candidate - curr
    };

    (distance, overflow)
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
        // Safe as december and january both have the same number of days
        return next_year(from).replace_month(Month::January).unwrap();
    }

    // Accelerator for days that exist in all months
    if from.day() < 28 {
        return from.add(Duration::days(days_in_current_month(from).into()));
    }

    month_with_day(from, from.day(), false)
}

fn month_with_day(from: OffsetDateTime, from_day: u8, try_current_month: bool) -> OffsetDateTime {
    // Safe as all months have a '1' day
    let mut next = if try_current_month {
        from
    } else {
        next_month(from.replace_day(1).unwrap())
    };

    while days_in_current_month(next) < from_day {
        // Safe as all months have a '1' day
        next = next_month(next.replace_day(1).unwrap());
    }

    // Safe because we just checked that the current month had enough days for this
    next.replace_day(from_day).unwrap()
}

fn next_year(from: OffsetDateTime) -> OffsetDateTime {
    let mut inc = 0;

    loop {
        match from.replace_year(from.year() + inc) {
            Ok(date) => return date,
            Err(_) => {
                inc += 1;
            }
        }
    }
}
