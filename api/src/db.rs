use std::path::Path;

use anyhow::Result;
use mello::kvstorage::KVStorage;

use crate::invitation::{Invitation, InvitationCode};

// options
const FIRST_SIGNUP_INVITATION: &str = "option:first-signup-invitation";

#[derive(Clone)]
pub struct Database {
    storage: KVStorage,
}

impl Database {
    /// Create a new instance of  in-memory database.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let storage = KVStorage::open(path)?;
        Ok(Self { storage })
    }

    /// Create the first signup invitation used to register the administrator.
    pub fn create_first_signup_invitation(&self, username: &str) -> Result<Option<InvitationCode>> {
        let mut conn = self.storage.write();
        let tx = conn.transaction()?;

        if tx.get(FIRST_SIGNUP_INVITATION)?.unwrap_or(false) {
            return Ok(None);
        }

        let invitation_code = InvitationCode::random();
        let invitation = Invitation::new(username);

        tx.set(
            format!("invitation:{}", invitation_code.display()),
            &invitation,
        )?;
        tx.set(FIRST_SIGNUP_INVITATION, &true)?;
        tx.commit()?;

        Ok(Some(invitation_code))
    }

    /// Get the invitation from its code.
    pub fn get_signup_invitation(&self, code: &InvitationCode) -> Result<Option<Invitation>> {
        let conn = self.storage.read()?;
        Ok(conn
            .get::<_, Invitation>(format!("invitation:{}", code.display()))?
            .filter(|invitation| !invitation.is_expired()))
    }

    /// Create a new signup invitation.
    pub fn create_signup_invitation(&self, username: &str) -> Result<InvitationCode> {
        let conn = self.storage.write();

        let invitation_code = InvitationCode::random();
        let invitation = Invitation::new(username);

        conn.set(
            format!("invitation:{}", invitation_code.display()),
            &invitation,
        )?;

        Ok(invitation_code)
    }
}
