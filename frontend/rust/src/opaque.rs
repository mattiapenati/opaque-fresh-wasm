use std::cell::RefCell;

use base64ct::{Base64Url, Encoding};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use wasm_bindgen::prelude::*;

thread_local! {
    static RNG: RefCell<ChaChaRng> = RefCell::new(ChaChaRng::from_entropy());
}

/// Cipher suite definition
pub struct CipherSuite;

impl opaque_ke::CipherSuite for CipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = argon2::Argon2<'static>;
}

#[wasm_bindgen(getter_with_clone)]
pub struct OpaqueRegistration {
    state: opaque_ke::ClientRegistration<CipherSuite>,
    /// Base64 encoded message should be sent to the server.
    pub message: String,
}

#[wasm_bindgen(getter_with_clone)]
pub struct OpaqueRegistrationFinish {
    /// The registration upload message to be sent to the server
    pub message: String,
}

#[wasm_bindgen]
impl OpaqueRegistration {
    /// Start registration step.
    pub fn start(password: &str) -> Result<OpaqueRegistration, JsError> {
        let registration_start = RNG
            .with_borrow_mut(|rng| {
                opaque_ke::ClientRegistration::<CipherSuite>::start(rng, password.as_bytes())
            })
            .map_err(JsError::from)?;

        let message = registration_start.message.serialize();
        let message = Base64Url::encode_string(&message);

        Ok(OpaqueRegistration {
            state: registration_start.state,
            message,
        })
    }

    /// Finish the registration.
    pub fn finish(
        self,
        password: &str,
        message: &str,
    ) -> Result<OpaqueRegistrationFinish, JsError> {
        let registration_response = Base64Url::decode_vec(message).map_err(JsError::from)?;
        let registration_response =
            opaque_ke::RegistrationResponse::deserialize(&registration_response)
                .map_err(JsError::from)?;
        let params = opaque_ke::ClientRegistrationFinishParameters::default();

        let registration_finish = RNG
            .with_borrow_mut(|rng| {
                self.state
                    .finish(rng, password.as_bytes(), registration_response, params)
            })
            .map_err(JsError::from)?;

        let message = registration_finish.message.serialize();
        let message = Base64Url::encode_string(&message);

        Ok(OpaqueRegistrationFinish { message })
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct OpaqueLogin {
    state: opaque_ke::ClientLogin<CipherSuite>,
    /// Base64 encoded message should be sent to the server.
    pub message: String,
}

#[wasm_bindgen(getter_with_clone)]
pub struct OpaqueLoginFinish {
    /// The login upload message to be sent to the server
    pub message: String,
}

#[wasm_bindgen]
impl OpaqueLogin {
    /// Start login step.
    pub fn start(password: &str) -> Result<OpaqueLogin, JsError> {
        let login_start = RNG
            .with_borrow_mut(|rng| {
                opaque_ke::ClientLogin::<CipherSuite>::start(rng, password.as_bytes())
            })
            .map_err(JsError::from)?;

        let message = login_start.message.serialize();
        let message = Base64Url::encode_string(&message);

        Ok(OpaqueLogin {
            state: login_start.state,
            message,
        })
    }

    /// Finish the login.
    pub fn finish(self, password: &str, message: &str) -> Result<OpaqueLoginFinish, JsError> {
        let credential_response = Base64Url::decode_vec(message).map_err(JsError::from)?;
        let credential_response = opaque_ke::CredentialResponse::deserialize(&credential_response)
            .map_err(JsError::from)?;
        let params = opaque_ke::ClientLoginFinishParameters::default();

        let registration_finish = self
            .state
            .finish(password.as_bytes(), credential_response, params)
            .map_err(JsError::from)?;

        let message = registration_finish.message.serialize();
        let message = Base64Url::encode_string(&message);

        Ok(OpaqueLoginFinish { message })
    }
}
