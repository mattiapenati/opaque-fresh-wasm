use std::{cell::RefCell, time::Duration};

use base64ct::{Base64Url, Encoding};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::time::DateTime;

/// Invitation code.
///
/// Invitation code is a sequence of random bytes generated using a
/// cryptographically secure random number generator. Invitation codes can be
/// displayed using the base64 (url standard) encoding.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct InvitationCode(InvitationCodeBytes);

type InvitationCodeBytes = [u8; 32];

thread_local! {
    static RNG: RefCell<ChaChaRng> = RefCell::new(ChaChaRng::from_entropy());
}

impl InvitationCode {
    /// Generate a new random invitation code.
    pub fn random() -> InvitationCode {
        let mut bytes = InvitationCodeBytes::default();
        RNG.with_borrow_mut(|rng| rng.fill_bytes(&mut bytes));
        Self(bytes)
    }

    /// Returns an object for printing the invitation code.
    pub fn display(&self) -> DisplayInvitationCode<'_> {
        DisplayInvitationCode(&self.0)
    }
}

/// Helper struct for explicit printing a `InvitationCode`.
pub struct DisplayInvitationCode<'a>(&'a [u8]);

impl<'a> std::fmt::Display for DisplayInvitationCode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(bytes) = self;
        let encoded_code = Base64Url::encode_string(bytes);
        f.write_str(&encoded_code)
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
const INVITATION_LIFETIME: Duration = Duration::from_secs(24 * 3_600);

impl Invitation {
    pub fn new(username: &str) -> Self {
        let username = username.to_owned();
        let expiration = DateTime::now() + INVITATION_LIFETIME;

        Self {
            username,
            expiration,
        }
    }
}
