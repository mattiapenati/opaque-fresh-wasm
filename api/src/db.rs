use std::path::Path;

use anyhow::Result;

use crate::{
    invitation::{Invitation, InvitationCode},
    storage::KVStorage,
};

// options
const FIRST_SIGNUP_INVITATION: &str = "option:first-signup-invitation";

pub struct Database {
    storage: KVStorage,
}

impl Database {
    /// Create a new instance of  in-memory database.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let storage = KVStorage::new(path)?;
        Ok(Self { storage })
    }

    pub fn create_first_signup_invitation(&self, username: &str) -> Result<Option<InvitationCode>> {
        let mut write = self.storage.write();
        let tx = write.transaction()?;

        if tx.pull(FIRST_SIGNUP_INVITATION)?.unwrap_or(false) {
            return Ok(None);
        }

        let invitation_code = InvitationCode::random();
        let invitation = Invitation::new(username);

        tx.push(
            format!("invitation:{}", invitation_code.display()),
            &invitation,
        )?;
        tx.push(FIRST_SIGNUP_INVITATION, &true)?;
        tx.commit()?;

        Ok(Some(invitation_code))
    }
}
