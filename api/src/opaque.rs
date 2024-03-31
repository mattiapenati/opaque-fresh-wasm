use std::cell::RefCell;

use anyhow::Result;
use base64ct::{Base64Url, Encoding};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Cipher suite definition
pub struct CipherSuite;

impl opaque_ke::CipherSuite for CipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = argon2::Argon2<'static>;
}

thread_local! {
    static RNG: RefCell<ChaChaRng> = RefCell::new(ChaChaRng::from_entropy());
}

/// Server signature
pub struct OpaqueSignature {
    server_setup: opaque_ke::ServerSetup<CipherSuite>,
}

impl OpaqueSignature {
    /// Generate a new random server signature.
    pub fn generate_random() -> String {
        let server_setup = RNG.with_borrow_mut(opaque_ke::ServerSetup::<CipherSuite>::new);
        let signature = server_setup.serialize();
        Base64Url::encode_string(&signature)
    }

    /// Create a new signature.
    pub fn new(signature: &str) -> Result<Self> {
        let signature = Base64Url::decode_vec(signature)?;
        let server_setup = opaque_ke::ServerSetup::<CipherSuite>::deserialize(&signature)?;
        Ok(Self { server_setup })
    }
}

///
/// From the client's blinded password returns a response to be sent back to the client.
pub fn registration_start(
    signature: &OpaqueSignature,
    username: &str,
    request: RegistrationRequest,
) -> Result<RegistrationResponse> {
    let server_registration_start = opaque_ke::ServerRegistration::start(
        &signature.server_setup,
        request.message,
        username.as_bytes(),
    )?;

    let message = server_registration_start.message;

    Ok(RegistrationResponse { message })
}

/// Finish the registration process and generate a password file.
pub fn registration_finish(upload: RegistrationUpload) -> PasswordFile {
    let registration = opaque_ke::ServerRegistration::finish(upload.message);
    PasswordFile { registration }
}

/// From the client's bindled password returns a response to be sent back to the client.
pub fn login_start(
    signature: &OpaqueSignature,
    username: &str,
    password_file: Option<PasswordFile>,
    request: LoginRequest,
) -> Result<(LoginResponse, LoginState)> {
    let params = opaque_ke::ServerLoginStartParameters::default();
    let registration = password_file.map(|pf| pf.registration);
    let credential_request = request.message;

    let server_login = RNG.with_borrow_mut(|rng| {
        opaque_ke::ServerLogin::start(
            rng,
            &signature.server_setup,
            registration,
            credential_request,
            username.as_bytes(),
            params,
        )
    })?;

    let login_response = LoginResponse {
        message: server_login.message,
    };
    let login_state = LoginState {
        state: server_login.state,
    };

    Ok((login_response, login_state))
}

/// Check the client's authentication
pub fn login_finish(state: LoginState, message: LoginFinalization) -> Result<()> {
    state.state.finish(message.message)?;
    Ok(())
}

/// Client registration request
pub struct RegistrationRequest {
    message: opaque_ke::RegistrationRequest<CipherSuite>,
}

impl<'de> Deserialize<'de> for RegistrationRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded_message: &str = Deserialize::deserialize(deserializer)?;
        let buffer = Base64Url::decode_vec(&encoded_message).map_err(serde::de::Error::custom)?;
        let message = opaque_ke::RegistrationRequest::deserialize(&buffer)
            .map_err(serde::de::Error::custom)?;
        Ok(Self { message })
    }
}

/// Server registration response
pub struct RegistrationResponse {
    message: opaque_ke::RegistrationResponse<CipherSuite>,
}

impl Serialize for RegistrationResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serialized_state = self.message.serialize();
        let encoded_state = Base64Url::encode_string(&serialized_state);
        serializer.serialize_str(&encoded_state)
    }
}

/// Client final registration request.
pub struct RegistrationUpload {
    message: opaque_ke::RegistrationUpload<CipherSuite>,
}

impl<'de> Deserialize<'de> for RegistrationUpload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded_message: &str = Deserialize::deserialize(deserializer)?;
        let buffer = Base64Url::decode_vec(&encoded_message).map_err(serde::de::Error::custom)?;
        let message = opaque_ke::RegistrationUpload::deserialize(&buffer)
            .map_err(serde::de::Error::custom)?;
        Ok(Self { message })
    }
}

/// Client login request
pub struct LoginRequest {
    message: opaque_ke::CredentialRequest<CipherSuite>,
}

impl<'de> Deserialize<'de> for LoginRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded_message: &str = Deserialize::deserialize(deserializer)?;
        let buffer = Base64Url::decode_vec(&encoded_message).map_err(serde::de::Error::custom)?;
        let message =
            opaque_ke::CredentialRequest::deserialize(&buffer).map_err(serde::de::Error::custom)?;
        Ok(Self { message })
    }
}

/// Server login response
pub struct LoginResponse {
    message: opaque_ke::CredentialResponse<CipherSuite>,
}

impl Serialize for LoginResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serialized_message = self.message.serialize();
        let encoded_message = Base64Url::encode_string(&serialized_message);
        serializer.serialize_str(&encoded_message)
    }
}

/// Client final login request.
pub struct LoginFinalization {
    message: opaque_ke::CredentialFinalization<CipherSuite>,
}

impl<'de> Deserialize<'de> for LoginFinalization {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded_message: &str = Deserialize::deserialize(deserializer)?;
        let buffer = Base64Url::decode_vec(&encoded_message).map_err(serde::de::Error::custom)?;
        let message = opaque_ke::CredentialFinalization::deserialize(&buffer)
            .map_err(serde::de::Error::custom)?;
        Ok(Self { message })
    }
}

/// Server login state
pub struct LoginState {
    state: opaque_ke::ServerLogin<CipherSuite>,
}

impl<'de> Deserialize<'de> for LoginState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded_message: &str = Deserialize::deserialize(deserializer)?;
        let buffer = Base64Url::decode_vec(&encoded_message).map_err(serde::de::Error::custom)?;
        let state =
            opaque_ke::ServerLogin::deserialize(&buffer).map_err(serde::de::Error::custom)?;
        Ok(Self { state })
    }
}

impl Serialize for LoginState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serialized_state = self.state.serialize();
        let encoded_state = Base64Url::encode_string(&serialized_state);
        serializer.serialize_str(&encoded_state)
    }
}

/// User registration password file.
pub struct PasswordFile {
    registration: opaque_ke::ServerRegistration<CipherSuite>,
}

impl<'de> Deserialize<'de> for PasswordFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded_message: &str = Deserialize::deserialize(deserializer)?;
        let buffer = Base64Url::decode_vec(&encoded_message).map_err(serde::de::Error::custom)?;
        let registration = opaque_ke::ServerRegistration::deserialize(&buffer)
            .map_err(serde::de::Error::custom)?;
        Ok(Self { registration })
    }
}

impl Serialize for PasswordFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serialized_registration = self.registration.serialize();
        let encoded_registration = Base64Url::encode_string(&serialized_registration);
        serializer.serialize_str(&encoded_registration)
    }
}
