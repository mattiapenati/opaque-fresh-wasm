use base64ct::{Base64Url, Encoding};
use opaque_ke::ClientRegistration;
use rand::SeedableRng;
use wasm_bindgen::prelude::*;

/// Cipher suite definition
pub struct CipherSuite;

type KeGroup = opaque_ke::Ristretto255;

impl opaque_ke::CipherSuite for CipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = KeGroup;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = argon2::Argon2<'static>;
}

#[wasm_bindgen(getter_with_clone)]
pub struct OpaqueRegistration {
    rng: rand_chacha::ChaChaRng,
    state: opaque_ke::ClientRegistration<CipherSuite>,
    /// Base64 encoed message should be sent to the server.
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
        let mut rng = rand_chacha::ChaChaRng::from_entropy();
        let registration_start =
            ClientRegistration::<CipherSuite>::start(&mut rng, password.as_bytes())
                .map_err(JsError::from)?;

        let message = registration_start.message.serialize();
        let message = Base64Url::encode_string(&message);

        Ok(OpaqueRegistration {
            rng,
            state: registration_start.state,
            message,
        })
    }

    /// Finish the registration.
    pub fn finish(
        mut self,
        password: &str,
        message: &str,
    ) -> Result<OpaqueRegistrationFinish, JsError> {
        let registration_response = Base64Url::decode_vec(message).map_err(JsError::from)?;
        let registration_response =
            opaque_ke::RegistrationResponse::deserialize(&registration_response)
                .map_err(JsError::from)?;
        let params = opaque_ke::ClientRegistrationFinishParameters::default();

        let registration_finish = self
            .state
            .finish(
                &mut self.rng,
                password.as_bytes(),
                registration_response,
                params,
            )
            .map_err(JsError::from)?;

        let message = registration_finish.message.serialize();
        let message = Base64Url::encode_string(&message);

        Ok(OpaqueRegistrationFinish { message })
    }
}
