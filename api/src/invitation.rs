use std::cell::RefCell;

use base64ct::{Base64Url, Encoding};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Invitation code.
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
