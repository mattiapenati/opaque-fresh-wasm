use std::sync::Arc;

use mello::kvstorage::KVStorage;

use crate::opaque::OpaqueServer;

/// Application state
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    storage: KVStorage,
    opaque: OpaqueServer,
}

impl AppState {
    /// Create a new application state.
    pub fn new(storage: KVStorage, opaque: OpaqueServer) -> Self {
        let inner = Inner { storage, opaque };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn storage(&self) -> &KVStorage {
        &self.inner.storage
    }

    pub fn opaque(&self) -> &OpaqueServer {
        &self.inner.opaque
    }
}
