use std::sync::Arc;

use anyhow::Result;

use crate::{
    db::Database,
    invitation::{Invitation, InvitationCode},
};

/// Application state
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    db: Database,
}

impl AppState {
    /// Create a new application state.
    pub fn new(db: Database) -> Self {
        let inner = Inner { db };
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Get the invitation from the code.
    #[inline(always)]
    pub fn get_signup_invitation(&self, code: &InvitationCode) -> Result<Option<Invitation>> {
        self.inner.db.get_signup_invitation(code)
    }
}
