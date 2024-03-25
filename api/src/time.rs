use serde::{Deserialize, Serialize};

/// Combined date and time with an offset applied.
#[derive(Deserialize, Serialize)]
pub struct DateTime(#[serde(with = "time::serde::rfc3339")] time::OffsetDateTime);

impl DateTime {
    /// The current date and time.
    pub fn now() -> Self {
        Self(time::OffsetDateTime::now_utc())
    }
}

impl std::ops::Add<std::time::Duration> for DateTime {
    type Output = Self;

    fn add(self, rhs: std::time::Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::cmp::PartialEq for DateTime {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl std::cmp::Eq for DateTime {}

impl std::cmp::PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for DateTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
