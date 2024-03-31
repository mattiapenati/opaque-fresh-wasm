use std::sync::Arc;

use mello::kvstorage::KVStorage;

use crate::opaque::OpaqueSignature;

/// Application state
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    storage: KVStorage,
    signature: OpaqueSignature,
}

impl AppState {
    /// Create a new application state.
    pub fn new(storage: KVStorage, signature: OpaqueSignature) -> Self {
        let inner = Inner { storage, signature };
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Returns a reference to the storage.
    pub fn storage(&self) -> &KVStorage {
        &self.inner.storage
    }

    /// Returns a reference to the server signature.
    pub fn signature(&self) -> &OpaqueSignature {
        &self.inner.signature
    }
}
