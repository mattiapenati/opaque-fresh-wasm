use anyhow::Result;
use base64ct::{Base64, Base64Url, Encoding};
use generic_array::GenericArray;
use opaque_ke::keypair::SecretKey;
use rand::SeedableRng;

/// Cipher suite definition
pub struct CipherSuite;

type KeGroup = opaque_ke::Ristretto255;
type PrivateKey = opaque_ke::keypair::PrivateKey<KeGroup>;
type KeyPair = opaque_ke::keypair::KeyPair<KeGroup, PrivateKey>;

impl opaque_ke::CipherSuite for CipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = KeGroup;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = argon2::Argon2<'static>;
}

pub struct OpaqueServer {
    setup: opaque_ke::ServerSetup<CipherSuite>,
}

impl OpaqueServer {
    /// Generate a new base64 encoded random private key
    pub fn generate_random_key() -> String {
        let mut rng = rand_chacha::ChaChaRng::from_entropy();
        let server_setup = opaque_ke::ServerSetup::<CipherSuite>::new(&mut rng);
        let private_key = server_setup.keypair().private();
        let serialized_key = private_key.serialize();
        base64ct::Base64::encode_string(&serialized_key)
    }

    pub fn new(key: &str) -> Self {
        let serialized_key = Base64::decode_vec(key).expect("a base64 encoded private key");
        let private_key = PrivateKey::deserialize(&serialized_key).expect("a valid private key");
        let key_pair = KeyPair::from_private_key(private_key).expect("a valid key");
        let mut rng = rand_chacha::ChaChaRng::from_entropy();
        let setup = opaque_ke::ServerSetup::new_with_key(&mut rng, key_pair);
        Self { setup }
    }

    /// From the client's blinded password returns a response to be sent back to the client.
    pub fn registration_start(
        &self,
        username: &str,
        request: RegistrationRequest,
    ) -> Result<RegistrationResponse> {
        let server_registration_start = opaque_ke::ServerRegistration::start(
            &self.setup,
            request.message,
            username.as_bytes(),
        )?;

        let message = server_registration_start.message;

        Ok(RegistrationResponse { message })
    }

    /// Finish the registration process and generate a password file.
    pub fn registration_finish(&self, upload: RegistrationUpload) -> PasswordFile {
        let registration = opaque_ke::ServerRegistration::finish(upload.message);
        PasswordFile { registration }
    }
}

/// Client registration request
pub struct RegistrationRequest {
    message: opaque_ke::RegistrationRequest<CipherSuite>,
}

impl<'de> serde::Deserialize<'de> for RegistrationRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut buffer =
            GenericArray::<u8, opaque_ke::RegistrationRequestLen<CipherSuite>>::default();

        let encoded_message = <&str as serde::Deserialize>::deserialize(deserializer)?;
        Base64Url::decode(encoded_message, &mut buffer[..]).map_err(serde::de::Error::custom)?;

        let message = opaque_ke::RegistrationRequest::deserialize(&buffer)
            .map_err(serde::de::Error::custom)?;

        Ok(Self { message })
    }
}

/// Server registration response
pub struct RegistrationResponse {
    message: opaque_ke::RegistrationResponse<CipherSuite>,
}

impl serde::Serialize for RegistrationResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serialized_message = self.message.serialize();
        let encoded_message = Base64Url::encode_string(&serialized_message);
        serializer.serialize_str(&encoded_message)
    }
}

/// Client final registration request.
pub struct RegistrationUpload {
    message: opaque_ke::RegistrationUpload<CipherSuite>,
}

impl<'de> serde::Deserialize<'de> for RegistrationUpload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut buffer =
            GenericArray::<u8, opaque_ke::RegistrationUploadLen<CipherSuite>>::default();

        let encoded_message = <&str as serde::Deserialize>::deserialize(deserializer)?;
        Base64Url::decode(encoded_message, &mut buffer[..]).map_err(serde::de::Error::custom)?;

        let message = opaque_ke::RegistrationUpload::deserialize(&buffer)
            .map_err(serde::de::Error::custom)?;

        Ok(Self { message })
    }
}

/// User registration password file.
pub struct PasswordFile {
    registration: opaque_ke::ServerRegistration<CipherSuite>,
}

impl<'de> serde::Deserialize<'de> for PasswordFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut buffer =
            GenericArray::<u8, opaque_ke::ServerRegistrationLen<CipherSuite>>::default();

        let encoded_message = <&str as serde::Deserialize>::deserialize(deserializer)?;
        Base64Url::decode(encoded_message, &mut buffer[..]).map_err(serde::de::Error::custom)?;

        let registration = opaque_ke::ServerRegistration::deserialize(&buffer)
            .map_err(serde::de::Error::custom)?;

        Ok(Self { registration })
    }
}

impl serde::Serialize for PasswordFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serialized_registration = self.registration.serialize();
        let encoded_registration = Base64Url::encode_string(&serialized_registration);
        serializer.serialize_str(&encoded_registration)
    }
}
