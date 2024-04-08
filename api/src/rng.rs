use std::cell::UnsafeCell;

use rand::{
    rngs::{adapter::ReseedingRng, OsRng},
    SeedableRng,
};
use rand_chacha::ChaCha20Core;

/// Default cryptographically secure random number generator.
pub struct CryptoRng {
    inner: ReseedingRng<ChaCha20Core, OsRng>,
}

impl CryptoRng {
    /// Create a new random number generator.
    fn new() -> Self {
        let rng = ChaCha20Core::from_entropy();
        let threshold = 64 * 1024; // 64kB
        let inner = ReseedingRng::new(rng, threshold, OsRng);
        Self { inner }
    }
}

impl rand::RngCore for CryptoRng {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    #[inline(always)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest)
    }

    #[inline(always)]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.inner.try_fill_bytes(dest)
    }
}

impl rand::CryptoRng for CryptoRng {}

/// Calls `f` passing the random number generator to `f`, returns the result from `f`.
pub fn with_crypto_rng<F, T>(f: F) -> T
where
    F: FnOnce(&mut CryptoRng) -> T,
{
    thread_local! {
        static RNG: UnsafeCell<CryptoRng> = UnsafeCell::new(CryptoRng::new());
    }

    RNG.with(|rng| f(unsafe { &mut *rng.get() }))
}
