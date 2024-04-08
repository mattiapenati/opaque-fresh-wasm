use std::str::FromStr;

use anyhow::Result;
use base64ct::{Base64Url, Encoding};
use ed25519_dalek::{
    ed25519::signature::Signer, SecretKey, Signature, SigningKey, SECRET_KEY_LENGTH,
};
use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::time::{DateTime, Duration};

/// Invitation key, used to sign the [`Invitation`].
pub struct InvitationKey {
    key: SigningKey,
}

impl InvitationKey {
    /// Generate a new random invitation key.
    pub fn generate<R: CryptoRngCore>(rng: &mut R) -> Self {
        let mut key = SecretKey::default();
        rng.fill_bytes(&mut key);

        Self {
            key: SigningKey::from_bytes(&key),
        }
    }

    /// Sign an [`Invitation`] and generate a new [`InvitationCode`].
    pub fn sign(&self, invitation: &Invitation) -> InvitationCode {
        let invitation = serde_json::to_string(&invitation).unwrap();
        let signature = self.key.sign(invitation.as_bytes());
        InvitationCode::from_parts(&invitation, signature)
    }

    /// Verify and [`InvitationCode`] and return the [`Invitation`].
    pub fn verify(&self, code: &InvitationCode) -> Result<Invitation, InvalidInvitationCode> {
        let (invitation, signature) = code.into_parts()?;
        self.key
            .verify(invitation.as_bytes(), &signature)
            .map_err(|err| {
                tracing::error!("failed to verify invitation: {err}");
                InvalidInvitationCode
            })?;
        let invitation: Invitation = serde_json::from_str(&invitation).map_err(|err| {
            tracing::error!("invitation payload is not a valid json: {err}");
            InvalidInvitationCode
        })?;
        if invitation.is_expired() {
            tracing::error!("used expired invitation");
            return Err(InvalidInvitationCode);
        }
        Ok(invitation)
    }

    /// Returns an object for printing the invitation key.
    pub fn display(&self) -> DisplayInvitationKey<'_> {
        DisplayInvitationKey {
            bytes: self.key.as_bytes().as_slice(),
        }
    }
}

/// Helper struct for explicit printing a [`InvitationKey`].
pub struct DisplayInvitationKey<'a> {
    bytes: &'a [u8],
}

impl<'a> std::fmt::Display for DisplayInvitationKey<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut encoded_bytes = [0_u8; 44];
        let encoded_code = Base64Url::encode(self.bytes, &mut encoded_bytes).unwrap();
        f.write_str(encoded_code)
    }
}

/// An error which can be returned when parsing a [`InvitationKey`].
#[derive(Clone, Copy, Debug)]
pub struct InvalidInvitationKey;

impl std::fmt::Display for InvalidInvitationKey {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid invitation key")
    }
}

impl std::error::Error for InvalidInvitationKey {}

impl FromStr for InvitationKey {
    type Err = InvalidInvitationKey;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut key = SecretKey::default();
        let decoded_key =
            Base64Url::decode(s.as_bytes(), &mut key).map_err(|_| InvalidInvitationKey)?;
        if decoded_key.len() != SECRET_KEY_LENGTH {
            return Err(InvalidInvitationKey);
        }

        let key = SigningKey::from_bytes(&key);
        Ok(Self { key })
    }
}

/// Sign up invitation.
#[derive(Deserialize, Serialize)]
pub struct Invitation {
    /// Invited username (should match on registration).
    pub username: String,
    /// Expiration of the this invitation.
    #[serde(
        serialize_with = "DateTime::serialize_unix_timestamp",
        deserialize_with = "DateTime::deserialize_unix_timestamp"
    )]
    expiration: DateTime,
}

impl Invitation {
    /// Default invitation lifetime (1 day).
    const DEFAULT_LIFETIME: Duration = Duration::hours(24);

    /// Admin invitation lifetime (10 minutes).
    const ADMIN_LIFETIME: Duration = Duration::minutes(10);

    /// Create a new invitation for the user and with default lifetime.
    pub fn new(username: &str) -> Self {
        Self::with_lifetime(username, Self::DEFAULT_LIFETIME)
    }

    /// Create a new invitation for an administrator.
    pub fn admin(username: &str) -> Self {
        Self::with_lifetime(username, Self::ADMIN_LIFETIME)
    }

    /// Create a new invitation for the user and with the given lifetime.
    fn with_lifetime(username: &str, lifetime: Duration) -> Self {
        Self {
            username: username.to_string(),
            expiration: DateTime::now() + lifetime,
        }
    }

    /// Check if the invitation is expired.
    fn is_expired(&self) -> bool {
        self.expiration < DateTime::now()
    }
}

/// Invitation code.
///
/// Invitation code is composed by two parts encoded separately using base64
/// url encoding and concatenated using a period. The first part is the JSON
/// serialized [`Invitation`] and the second one is the signature produced using
/// the [`InvitationKey`].
#[derive(Debug, Deserialize, Serialize, Zeroize, ZeroizeOnDrop)]
#[serde(transparent)]
pub struct InvitationCode(String);

impl std::fmt::Display for InvitationCode {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl InvitationCode {
    /// Compose the invitation code from the parts.
    fn from_parts(invitation: &str, signature: Signature) -> Self {
        let invitation = Base64Url::encode_string(invitation.as_bytes());
        let signature = Base64Url::encode_string(signature.to_bytes().as_slice());

        Self(format!("{invitation}.{signature}"))
    }

    /// Split the invitation code into its parts.
    fn into_parts(&self) -> Result<(String, Signature), InvalidInvitationCode> {
        let (invitation, signature) = self.0.split_once('.').ok_or(InvalidInvitationCode)?;

        let invitation = Base64Url::decode_vec(invitation).map_err(|_| InvalidInvitationCode)?;
        let invitation = String::from_utf8(invitation).map_err(|_| InvalidInvitationCode)?;

        let mut bytes = [0u8; Signature::BYTE_SIZE];
        let signature_len = Base64Url::decode(signature.as_bytes(), &mut bytes)
            .map_err(|_| InvalidInvitationCode)?
            .len();
        if signature_len != Signature::BYTE_SIZE {
            dbg!(signature_len);
            dbg!(Signature::BYTE_SIZE);
            tracing::error!("inviation signature has wrong length");
            return Err(InvalidInvitationCode);
        }
        let signature = Signature::from_bytes(&bytes);

        Ok((invitation, signature))
    }
}

/// An error which can be returned when parsing a [`InvitationCode`].
#[derive(Clone, Copy, Debug)]
pub struct InvalidInvitationCode;
