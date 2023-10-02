// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::{DateTime, Duration, TimeZone, Utc};
use std::borrow::Cow;

macro_rules! plural {
    ($n:expr, $singular:expr) => {
        $crate::util::plural::Plural::from($singular).compute($n)
    };
}

macro_rules! rough_plural {
    ($n:expr, $singular:expr, $article:expr) => {
        $crate::util::plural::Rough::from($singular)
            .article($article)
            .compute($n)
    };
    ($n:expr, $singular:expr) => {
        $crate::util::plural::Rough::from($singular).compute($n)
    };
}

pub trait HumanFriendly {
    fn human_friendly(&self) -> String;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum Tense {
    Past,
    Present,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum Accuracy {
    Rough,
    Precise,
}

impl Accuracy {
    pub fn is_rough(self) -> bool {
        self == Self::Rough
    }
}

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;
const MONTH: i64 = 30 * DAY;
const YEAR: i64 = 365 * DAY;

#[derive(Clone, Copy, Debug)]
enum TimePeriod {
    Now,
    Seconds(i64),
    Minutes(i64),
    Hours(i64),
    Days(i64),
    Weeks(i64),
    Months(i64),
    Years(i64),
    Eternity,
}

impl TimePeriod {
    fn to_text_precise(self) -> Cow<'static, str> {
        match self {
            Self::Now => "now".into(),
            Self::Seconds(n) => plural!(n, "second").into(),
            Self::Minutes(n) => plural!(n, "minute").into(),
            Self::Hours(n) => plural!(n, "hour").into(),
            Self::Days(n) => plural!(n, "day").into(),
            Self::Weeks(n) => plural!(n, "week").into(),
            Self::Months(n) => plural!(n, "month").into(),
            Self::Years(n) => plural!(n, "year").into(),
            Self::Eternity => "eternity".into(),
        }
    }

    fn to_text_rough(self) -> Cow<'static, str> {
        match self {
            Self::Now => "now".into(),
            Self::Seconds(n) => rough_plural!(n, "second").into(),
            Self::Minutes(n) => rough_plural!(n, "minute").into(),
            Self::Hours(n) => rough_plural!(n, "hour", "an").into(),
            Self::Days(n) => rough_plural!(n, "day").into(),
            Self::Weeks(n) => rough_plural!(n, "week").into(),
            Self::Months(n) => rough_plural!(n, "month").into(),
            Self::Years(n) => rough_plural!(n, "year").into(),
            Self::Eternity => "eternity".into(),
        }
    }

    fn to_text(self, accuracy: Accuracy) -> Cow<'static, str> {
        match accuracy {
            Accuracy::Rough => self.to_text_rough(),
            Accuracy::Precise => self.to_text_precise(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub struct HumanTime(Duration);

impl HumanTime {
    const DAYS_IN_MONTH: i64 = 30;
    const DAYS_IN_YEAR: i64 = 365;

    pub fn to_text(self, accuracy: Accuracy, tense: Tense) -> String {
        let mut periods = match accuracy {
            Accuracy::Rough => self.rough_periods(),
            Accuracy::Precise => self.precise_periods(),
        };
        let to_tense = |text: Cow<'_, str>| match tense {
            Tense::Past => format!("{text} ago"),
            Tense::Present => text.into_owned(),
        };
        let first = periods.remove(0).to_text(accuracy);
        let last = periods.pop().map(|p| p.to_text(accuracy));
        if periods.is_empty() {
            if let Some(last) = last {
                return to_tense(Cow::Owned(format!("{first} and {last}")));
            }
            return to_tense(first);
        }
        let mut text = periods
            .into_iter()
            .fold(first, |acc, p| format!("{}, {}", acc, p.to_text(accuracy)).into());
        // this is always true, so we could just unwrap/expect
        // but I'm pretty sure it's optimized out anyway
        if let Some(last) = last {
            text = format!("{text}, and {last}").into();
        }
        to_tense(text)
    }

    fn tense(self, accuracy: Accuracy) -> Tense {
        if accuracy.is_rough() && self.0.num_seconds().abs() < 11 {
            Tense::Present
        } else if self.0 < Duration::zero() {
            Tense::Past
        } else {
            Tense::Present
        }
    }

    fn rough_periods(self) -> Vec<TimePeriod> {
        let period = match self.0.num_seconds().abs() {
            n if n > 547 * DAY => TimePeriod::Years(i64::max(n / YEAR, 2)), // ~1.5y
            n if n > 345 * DAY => TimePeriod::Years(1),                     // ~11m
            n if n > 45 * DAY => TimePeriod::Months(i64::max(n / MONTH, 2)), // ~1.5m
            n if n > 29 * DAY => TimePeriod::Months(1),
            n if n > 10 * DAY + 12 * HOUR => TimePeriod::Weeks(i64::max(n / WEEK, 2)),
            n if n > 6 * DAY + 12 * HOUR => TimePeriod::Weeks(1),
            n if n > 36 * HOUR => TimePeriod::Days(i64::max(n / DAY, 2)),
            n if n > 22 * HOUR => TimePeriod::Days(1),
            n if n > 90 * MINUTE => TimePeriod::Hours(i64::max(n / HOUR, 2)),
            n if n > 45 * MINUTE => TimePeriod::Hours(1),
            n if n > 90 => TimePeriod::Minutes(i64::max(n / MINUTE, 2)),
            n if n > 45 => TimePeriod::Minutes(1),
            n if n > 10 => TimePeriod::Seconds(n),
            0..=10 => TimePeriod::Now,
            _ => TimePeriod::Eternity,
        };
        vec![period]
    }

    fn precise_periods(self) -> Vec<TimePeriod> {
        let mut periods = vec![];
        let (years, remainder) = self.split_years();
        if let Some(years) = years {
            periods.push(TimePeriod::Years(years));
        }
        let (months, remainder) = remainder.split_months();
        if let Some(months) = months {
            periods.push(TimePeriod::Months(months));
        }
        let (weeks, remainder) = remainder.split_weeks();
        if let Some(weeks) = weeks {
            periods.push(TimePeriod::Weeks(weeks));
        }
        let (days, remainder) = remainder.split_days();
        if let Some(days) = days {
            periods.push(TimePeriod::Days(days));
        }
        let (hours, remainder) = remainder.split_hours();
        if let Some(hours) = hours {
            periods.push(TimePeriod::Hours(hours));
        }
        let (minutes, remainder) = remainder.split_minutes();
        if let Some(minutes) = minutes {
            periods.push(TimePeriod::Minutes(minutes));
        }
        let (seconds, _) = remainder.split_seconds();
        if let Some(seconds) = seconds {
            periods.push(TimePeriod::Seconds(seconds));
        }
        if periods.is_empty() {
            periods.push(TimePeriod::Seconds(0));
        }
        periods
    }

    fn normalize_split(wholes: impl Into<Option<i64>>, remainder: Duration) -> (Option<i64>, Self) {
        (wholes.into().map(i64::abs).filter(|x| *x > 0), Self(remainder))
    }

    fn split_years(self) -> (Option<i64>, Self) {
        let years = self.0.num_days() / Self::DAYS_IN_YEAR;
        let remainder = self.0 - Duration::days(years * Self::DAYS_IN_YEAR);
        Self::normalize_split(years, remainder)
    }

    fn split_months(self) -> (Option<i64>, Self) {
        let months = self.0.num_days() / Self::DAYS_IN_MONTH;
        let remainder = self.0 - Duration::days(months * Self::DAYS_IN_MONTH);
        Self::normalize_split(months, remainder)
    }

    fn split_weeks(self) -> (Option<i64>, Self) {
        let weeks = self.0.num_weeks();
        let remainder = self.0 - Duration::weeks(weeks);
        Self::normalize_split(weeks, remainder)
    }

    fn split_days(self) -> (Option<i64>, Self) {
        let days = self.0.num_days();
        let remainder = self.0 - Duration::days(days);
        Self::normalize_split(days, remainder)
    }

    fn split_hours(self) -> (Option<i64>, Self) {
        let hours = self.0.num_hours();
        let remainder = self.0 - Duration::hours(hours);
        Self::normalize_split(hours, remainder)
    }

    fn split_minutes(self) -> (Option<i64>, Self) {
        let minutes = self.0.num_minutes();
        let remainder = self.0 - Duration::minutes(minutes);
        Self::normalize_split(minutes, remainder)
    }

    fn split_seconds(self) -> (Option<i64>, Self) {
        let seconds = self.0.num_seconds();
        let remainder = self.0 - Duration::seconds(seconds);
        Self::normalize_split(seconds, remainder)
    }
}

impl std::fmt::Display for HumanTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let accuracy = if f.alternate() {
            Accuracy::Precise
        } else {
            Accuracy::Rough
        };
        f.pad(&self.to_text(accuracy, self.tense(accuracy)))
    }
}

impl From<Duration> for HumanTime {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

impl<TZ: TimeZone> From<DateTime<TZ>> for HumanTime {
    fn from(value: DateTime<TZ>) -> Self {
        value.signed_duration_since(Utc::now()).into()
    }
}

impl HumanFriendly for Duration {
    fn human_friendly(&self) -> String {
        format!("{}", HumanTime::from(*self))
    }
}

impl<TZ: TimeZone> HumanFriendly for DateTime<TZ> {
    fn human_friendly(&self) -> String {
        format!("{}", HumanTime::from(self.clone()))
    }
}
