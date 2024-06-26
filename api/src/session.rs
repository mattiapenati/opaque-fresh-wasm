use std::{cell::RefCell, str::FromStr};

use anyhow::ensure;
use base64ct::{Base64Url, Encoding};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;

const SESSION_BYTES: usize = 96;
const ENCODED_BYTES: usize = 128;

/// Generic session identifier.
pub struct SessionId {
    bytes: [u8; SESSION_BYTES],
}

impl SessionId {
    /// Generate a random session id.
    pub fn random() -> Self {
        thread_local! {
             static RNG: RefCell<ChaChaRng> = RefCell::new(ChaChaRng::from_entropy());
        }

        let mut bytes = [0_u8; SESSION_BYTES];
        RNG.with_borrow_mut(|rng| rng.fill_bytes(&mut bytes[..]));
        Self { bytes }
    }

    /// Returns an object for printing the invitation code.
    pub fn display(&self) -> DisplaySessionId<'_> {
        DisplaySessionId { bytes: &self.bytes }
    }

    /// Serializer function
    pub fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buffer = [0u8; ENCODED_BYTES];
        let display = Base64Url::encode(&self.bytes, &mut buffer).unwrap();
        serializer.serialize_str(display)
    }
}

/// Helper struct for explicit printing a `SessionId`.
pub struct DisplaySessionId<'a> {
    bytes: &'a [u8],
}

impl<'a> std::fmt::Display for DisplaySessionId<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut encoded_bytes = [0_u8; ENCODED_BYTES];
        let encoded_code = Base64Url::encode(self.bytes, &mut encoded_bytes).unwrap();
        f.write_str(encoded_code)
    }
}

impl FromStr for SessionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0_u8; SESSION_BYTES];
        let decoded_bytes = Base64Url::decode(s, &mut bytes)?.len();
        ensure!(
            decoded_bytes == SESSION_BYTES,
            format!("expected a string with {SESSION_BYTES} bytes base64 encoded")
        );

        Ok(Self { bytes })
    }
}

impl<'de> serde::Deserialize<'de> for SessionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let encoded_bytes: &str = serde::Deserialize::deserialize(deserializer)?;
        encoded_bytes.parse().map_err(serde::de::Error::custom)
    }
}
