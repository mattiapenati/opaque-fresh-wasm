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
