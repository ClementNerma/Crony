use std::num::ParseIntError;

use anyhow::{bail, Context, Result};
use once_cell::sync::Lazy;
use pomsky_macro::pomsky;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

static AT_STR_PARSER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(pomsky!(
        let sep = ^ | ' ';
        let every = '*' | [digit]+ (',' [digit]+)*;

        Start
        (sep "M=" :months(every))?
        (sep "D=" :days(every))?
        (sep "h=" :hours(every))?
        (sep "m=" :minutes(every))?
        (sep "s=" :seconds(every))?
        End
    ))
    .unwrap()
});

#[derive(Serialize, Deserialize)]
pub struct At {
    pub hours: Occurrences,
    pub minutes: Occurrences,
    pub seconds: Occurrences,
    pub days: Occurrences,
    pub months: Occurrences,
}

impl At {
    pub fn parse(at: &str) -> Result<Self> {
        let capture = AT_STR_PARSER
            .captures(at)
            .context("Invalid at format provided")?;

        let months = Self::validate_captured(&capture, "months", 12)?;
        let days = Self::validate_captured(&capture, "days", 31)?;
        let hours = Self::validate_captured(&capture, "hours", 23)?;
        let minutes = Self::validate_captured(&capture, "minutes", 59)?;
        let seconds = Self::validate_captured(&capture, "seconds", 59)?;

        if months.is_none()
            && days.is_none()
            && hours.is_none()
            && minutes.is_none()
            && seconds.is_none()
        {
            bail!("Please provide at least one time specifier");
        }

        Ok(Self {
            seconds: seconds.unwrap_or_else(|| {
                if months.is_some() || days.is_some() || hours.is_some() || minutes.is_some() {
                    Occurrences::First
                } else {
                    Occurrences::Every
                }
            }),
            minutes: minutes.unwrap_or_else(|| {
                if months.is_some() || days.is_some() || hours.is_some() {
                    Occurrences::First
                } else {
                    Occurrences::Every
                }
            }),
            hours: hours.unwrap_or_else(|| {
                if months.is_some() || days.is_some() {
                    Occurrences::First
                } else {
                    Occurrences::Every
                }
            }),
            days: days.unwrap_or_else(|| {
                if months.is_some() {
                    Occurrences::First
                } else {
                    Occurrences::Every
                }
            }),
            months: months.unwrap_or(Occurrences::Every),
        })
    }

    pub fn unparse(&self) -> String {
        let mut out = vec![];

        if let Some(months) = self.months.unparse() {
            out.push(format!("M={}", months));
        }

        if let Some(days) = self.days.unparse() {
            out.push(format!("D={}", days));
        }

        if let Some(hours) = self.hours.unparse() {
            out.push(format!("h={}", hours));
        }

        if let Some(minutes) = self.minutes.unparse() {
            out.push(format!("m={}", minutes));
        }

        if let Some(seconds) = self.seconds.unparse() {
            out.push(format!("s={}", seconds));
        }

        out.join(" ")
    }

    fn validate_captured(
        capture: &Captures,
        name: &'static str,
        max: u8,
    ) -> Result<Option<Occurrences>> {
        let group = match capture.name(name) {
            Some(group) => group.as_str(),
            None => return Ok(None),
        };

        let occ = Occurrences::parse(group).unwrap();

        let validate = |value: u8| {
            if value > max {
                bail!("The value provided for group '{}' is too high: maximum allowed is {}, found {}", name, max, value)
            } else {
                Ok(())
            }
        };

        match occ {
            Occurrences::Every => {}
            Occurrences::First => unreachable!(),
            Occurrences::Once(ref value) => validate(*value)?,
            Occurrences::Multiple(ref values) => {
                for value in values {
                    validate(*value)?;
                }
            }
        }

        Ok(Some(occ))
    }
}

#[derive(Serialize, Deserialize)]
pub enum Occurrences {
    Every,
    First,
    Once(u8),
    Multiple(Vec<u8>),
}

impl Occurrences {
    pub fn parse(str: &str) -> Result<Self> {
        if str == "*" {
            Ok(Self::Every)
        } else if !str.contains(',') {
            Ok(Self::Once(str::parse(str)?))
        } else {
            Ok(Self::Multiple(
                str.split(',')
                    .map(str::parse)
                    .collect::<Result<Vec<_>, ParseIntError>>()?,
            ))
        }
    }

    pub fn unparse(&self) -> Option<String> {
        match self {
            Self::Every => Some("*".to_string()),
            Self::First => None,
            Self::Once(num) => Some(num.to_string()),
            Self::Multiple(nums) => Some(
                nums.iter()
                    .map(|num| num.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        }
    }
}
