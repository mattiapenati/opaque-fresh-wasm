use std::{cmp::Ordering, ops::Add};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Combined date and time with an offset applied.
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct DateTime(#[serde(with = "time::serde::rfc3339")] time::OffsetDateTime);

/// A span of time with nanoseconds precision.
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Duration(time::Duration);

impl Duration {
    /// Zero duration.
    pub const ZERO: Duration = Duration(time::Duration::ZERO);

    /// Create a new `Duration` with the given number of days.
    pub const fn days(days: i64) -> Self {
        Self(time::Duration::days(days))
    }

    /// Create a new `Duration` with the given number of hours.
    pub const fn hours(hours: i64) -> Self {
        Self(time::Duration::hours(hours))
    }

    /// Create a new `Duration` with the given number of minutes.
    pub const fn minutes(minutes: i64) -> Self {
        Self(time::Duration::minutes(minutes))
    }
}

impl DateTime {
    /// The current date and time.
    pub fn now() -> Self {
        Self(time::OffsetDateTime::now_utc())
    }

    /// Returns the amount of time elapsed.
    pub fn duration_since(&self, earlier: DateTime) -> Duration {
        Duration(self.0 - earlier.0)
    }

    /// Serialize to unix timestamp, in millisecods for JS compatibility.
    pub fn serialize_unix_timestamp<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let unix_timestamp = self.0.unix_timestamp_nanos() / 1_000_000;
        serializer.serialize_i64(unix_timestamp as i64)
    }

    /// Deserialize from unix timestamp, in milliseconds for JS compatibility.
    pub fn deserialize_unix_timestamp<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let unix_timestamp = i64::deserialize(deserializer)? as i128;
        let dt =
            time::OffsetDateTime::from_unix_timestamp_nanos(unix_timestamp * 1_000_000).unwrap();
        Ok(Self(dt))
    }
}

impl Add<Duration> for DateTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl PartialEq for DateTime {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for DateTime {}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DateTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Duration {}

impl PartialOrd for Duration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Duration {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl From<Duration> for time::Duration {
    fn from(value: Duration) -> Self {
        value.0
    }
}
