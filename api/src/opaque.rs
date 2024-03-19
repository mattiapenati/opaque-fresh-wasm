use base64ct::{Base64, Encoding};
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
}
