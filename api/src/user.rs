//! User management

use anyhow::{anyhow, Result};
use cookie::Cookie;
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
const SESSION: &str = "session";
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
    let invitation = storage.read()?.get::<_, Invitation>(&key)?;

    // keep the storage clean
    if matches!(&invitation, Some(invitation) if invitation.is_expired()) {
        storage.write().del(&key)?;
        return Ok(None);
    }

    Ok(invitation)
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
    pub username: String,
    pub state: LoginState,
    created_at: DateTime,
}

impl SigninSession {
    const LIFETIME: Duration = Duration::minutes(1);

    /// Create a new signin session with the given data.
    pub fn new(username: String, state: LoginState) -> Self {
        Self {
            username,
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

#[derive(Deserialize, Serialize)]
pub struct Session {
    pub username: String,
    created_at: DateTime,
}

impl Session {
    pub const LIFETIME: Duration = Duration::days(7);
    pub const COOKIE: &'static str = "SESSIONID";

    /// Check if the session is expired.
    fn is_expired(&self) -> bool {
        DateTime::now().duration_since(self.created_at) > Self::LIFETIME
    }

    /// Create the session cookie.
    fn create_cookie(session_id: SessionId) -> Cookie<'static> {
        let value = session_id.display().to_string();
        Cookie::build((Self::COOKIE, value))
            .secure(true)
            .http_only(true)
            .same_site(cookie::SameSite::Strict)
            .path("/")
            .max_age(Self::LIFETIME.into())
            .build()
    }

    /// Remove cookie.
    fn remove_cookie() -> Cookie<'static> {
        Cookie::build(Self::COOKIE)
            .path("/")
            .max_age(Duration::ZERO.into())
            .build()
    }
}

/// Start a new session and return the cookie that should be set by the client.
pub fn start_new_session(storage: &KVStorage, username: String) -> Result<Cookie<'static>> {
    let session_id = SessionId::random();
    let session = Session {
        username,
        created_at: DateTime::now(),
    };

    let key = format!("{SESSION}:{}", session_id.display());
    storage.write().set(key, &session)?;

    Ok(Session::create_cookie(session_id))
}

/// End the session and return the cookie that should be set by the client.
pub fn finish_session(storage: &KVStorage, session_id: SessionId) -> Result<Cookie<'static>> {
    let key = format!("{SESSION}:{}", session_id.display());
    storage.write().del(key)?;
    Ok(Session::remove_cookie())
}

/// Retrieve the session.
pub fn get_session(storage: &KVStorage, session_id: SessionId) -> Result<Option<Session>> {
    let key = format!("{SESSION}:{}", session_id.display());
    let session = storage
        .read()?
        .get::<_, Session>(key)?
        .filter(|session| !session.is_expired());

    Ok(session)
}
