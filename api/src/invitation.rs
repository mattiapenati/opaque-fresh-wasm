use std::cell::RefCell;

use anyhow::Result;
use base64ct::{Base64Url, Encoding};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::time::{DateTime, Duration};

/// Invitation code.
///
/// Invitation code is a sequence of random bytes generated using a
/// cryptographically secure random number generator. Invitation codes can be
/// displayed using the base64 (url standard) encoding.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct InvitationCode {
    bytes: InvitationCodeBytes,
}

const INVITATION_CODE_BYTES: usize = 48;
const INVITATION_CODE_ENCODED: usize = 64;

type InvitationCodeBytes = [u8; INVITATION_CODE_BYTES];

thread_local! {
    static RNG: RefCell<ChaChaRng> = RefCell::new(ChaChaRng::from_entropy());
}

impl InvitationCode {
    /// Generate a new random invitation code.
    pub fn random() -> InvitationCode {
        let mut bytes = [0_u8; INVITATION_CODE_BYTES];
        RNG.with_borrow_mut(|rng| rng.fill_bytes(&mut bytes));
        Self { bytes }
    }

    /// Returns an object for printing the invitation code.
    pub fn display(&self) -> DisplayInvitationCode<'_> {
        DisplayInvitationCode { bytes: &self.bytes }
    }

    /// Serializer function
    pub fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut encoded_bytes = [0_u8; INVITATION_CODE_ENCODED];
        let code = Base64Url::encode(&self.bytes, &mut encoded_bytes).unwrap();
        serializer.serialize_str(code)
    }
}

impl<'de> Deserialize<'de> for InvitationCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let encoded_code: &str = Deserialize::deserialize(deserializer)?;
        let mut bytes = [0_u8; INVITATION_CODE_BYTES];
        Base64Url::decode(encoded_code.as_bytes(), &mut bytes[..])
            .map_err(serde::de::Error::custom)?;

        Ok(Self { bytes })
    }
}

/// Helper struct for explicit printing a `InvitationCode`.
pub struct DisplayInvitationCode<'a> {
    bytes: &'a [u8],
}

impl<'a> std::fmt::Display for DisplayInvitationCode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut encoded_bytes = [0_u8; INVITATION_CODE_ENCODED];
        let encoded_code = Base64Url::encode(self.bytes, &mut encoded_bytes).unwrap();
        f.write_str(encoded_code)
    }
}

/// Signup invitation.
#[derive(Deserialize, Serialize)]
pub struct Invitation {
    /// Invited username (should match on registration).
    pub username: String,
    /// Expiration of the this invitation.
    pub expiration: DateTime,
}

/// Default invitation lifetime (1 day).
const INVITATION_LIFETIME: Duration = Duration::hours(24);

impl Invitation {
    pub fn new(username: &str) -> Self {
        let username = username.to_owned();
        let expiration = DateTime::now() + INVITATION_LIFETIME;

        Self {
            username,
            expiration,
        }
    }

    /// Check if the invitation is expired.
    pub fn is_expired(&self) -> bool {
        self.expiration < DateTime::now()
    }
}
