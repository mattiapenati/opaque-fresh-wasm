use std::sync::Arc;

use mello::kvstorage::KVStorage;

use crate::{invitation::InvitationKey, opaque::OpaqueSignature};

/// Application state
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    storage: KVStorage,
    signature: OpaqueSignature,
    invitation_key: InvitationKey,
}

impl AppState {
    /// Create a new application state.
    pub fn new(
        storage: KVStorage,
        signature: OpaqueSignature,
        invitation_key: InvitationKey,
    ) -> Self {
        let inner = Inner {
            storage,
            signature,
            invitation_key,
        };
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

    /// Returns a reference to the inviation key.
    pub fn invitation_key(&self) -> &InvitationKey {
        &self.inner.invitation_key
    }
}
