//! User management

use anyhow::{anyhow, Result};
use mello::kvstorage::KVStorage;
use serde::{Deserialize, Serialize};

use crate::{
    invitation::{Invitation, InvitationCode},
    opaque::{LoginState, PasswordFile},
    session::SessionId,
    time::{DateTime, Duration},
};

const FIRST_SIGNUP_INVITATION: &str = "option:first-signup-invitation";
const INVITATION: &str = "invitation";
const SIGNUP_SESSION: &str = "signup-session";
const SIGNIN_SESSION: &str = "signin-session";
const PASSWORD: &str = "password";

/// Create the first signup invitation used to register the administrator.
pub fn create_first_signup_invitation(
    storage: &KVStorage,
    username: &str,
) -> Result<Option<InvitationCode>> {
    let mut conn = storage.write();
    let tx = conn.transaction()?;

    if tx.get(FIRST_SIGNUP_INVITATION)?.unwrap_or(false) {
        return Ok(None);
    }

    let invitation_code = InvitationCode::random();
    let invitation = Invitation::new(username);
    let key = format!("{INVITATION}:{}", invitation_code.display());
    tx.set(key, &invitation)?;

    tx.set(FIRST_SIGNUP_INVITATION, &true)?;
    tx.commit()?;

    Ok(Some(invitation_code))
}

/// Retrieve a valid invitation using its code.
pub fn get_signup_invitation(
    storage: &KVStorage,
    code: &InvitationCode,
) -> Result<Option<Invitation>> {
    let key = format!("{INVITATION}:{}", code.display());
    Ok(storage
        .read()
        .and_then(|conn| conn.get::<_, Invitation>(key))?
        .filter(|invitation| !invitation.is_expired()))
}

/// Check if the invitation exists and associated to the given user.
pub fn signup_invitation_is_valid(
    storage: &KVStorage,
    code: &InvitationCode,
    username: &str,
) -> Result<bool> {
    let Some(invitation) = get_signup_invitation(storage, code)? else {
        return Ok(false);
    };
    Ok(invitation.username == username)
}

/// Sign up session
#[derive(Deserialize, Serialize)]
pub struct SignupSession {
    #[serde(serialize_with = "InvitationCode::serialize")]
    pub code: InvitationCode,
    created_at: DateTime,
}

impl SignupSession {
    const LIFETIME: Duration = Duration::minutes(1);

    /// Create a new signup session with the given data.
    pub fn new(code: InvitationCode) -> Self {
        Self {
            code,
            created_at: DateTime::now(),
        }
    }

    /// Check if the signup session is expired.
    fn is_expired(&self) -> bool {
        DateTime::now().duration_since(self.created_at) > Self::LIFETIME
    }
}

/// Push the signup session in the storage.
pub fn push_signup_session(storage: &KVStorage, session: SignupSession) -> Result<SessionId> {
    let session_id = SessionId::random();
    let key = format!("{SIGNUP_SESSION}:{}", session_id.display());
    storage.write().set(key, &session)?;
    Ok(session_id)
}

/// Pull the signup session from the storage.
pub fn pull_signup_session(
    storage: &KVStorage,
    session_id: SessionId,
) -> Result<Option<SignupSession>> {
    let key = format!("{SIGNUP_SESSION}:{}", session_id.display());
    let session = storage
        .write()
        .extract::<_, SignupSession>(key)?
        .filter(|session| !session.is_expired());
    Ok(session)
}

/// Sign in session
#[derive(Deserialize, Serialize)]
pub struct SigninSession {
    pub state: LoginState,
    created_at: DateTime,
}

impl SigninSession {
    const LIFETIME: Duration = Duration::minutes(1);

    /// Create a new signin session with the given data.
    pub fn new(state: LoginState) -> Self {
        Self {
            state,
            created_at: DateTime::now(),
        }
    }

    /// Check if the signin session is expired.
    fn is_expired(&self) -> bool {
        DateTime::now().duration_since(self.created_at) > Self::LIFETIME
    }
}

/// Push the signin session in the storage.
pub fn push_signin_session(storage: &KVStorage, session: SigninSession) -> Result<SessionId> {
    let session_id = SessionId::random();
    let key = format!("{SIGNIN_SESSION}:{}", session_id.display());
    storage.write().set(key, &session)?;
    Ok(session_id)
}
/// Pull the signin session from the storage.
pub fn pull_signin_session(
    storage: &KVStorage,
    session_id: SessionId,
) -> Result<Option<SigninSession>> {
    let key = format!("{SIGNIN_SESSION}:{}", session_id.display());
    let session = storage
        .write()
        .extract::<_, SigninSession>(key)?
        .filter(|session| !session.is_expired());
    Ok(session)
}

/// Register a new user, removing the used invitation.
pub fn register_user_password(
    storage: &KVStorage,
    code: &InvitationCode,
    password_file: PasswordFile,
) -> Result<()> {
    let mut conn = storage.write();
    let tx = conn.transaction()?;

    let key = format!("{INVITATION}:{}", code.display());
    let invitation = tx
        .extract::<_, Invitation>(key)?
        .ok_or_else(|| anyhow!("invitation does not exist"))?;

    let key = format!("{PASSWORD}:{}", invitation.username);
    tx.set(key, &password_file)?;

    tx.commit()?;
    Ok(())
}

/// Register a new user, removing the used invitation.
pub fn get_password_file(storage: &KVStorage, username: &str) -> Result<Option<PasswordFile>> {
    let key = format!("{PASSWORD}:{}", username);
    let password_file = storage.read()?.get(key)?;
    Ok(password_file)
}
