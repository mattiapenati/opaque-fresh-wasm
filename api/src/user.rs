//! User management

use anyhow::Result;
use cookie::Cookie;
use mello::kvstorage::KVStorage;
use serde::{Deserialize, Serialize};

use crate::{
    opaque::{LoginState, PasswordFile},
    session::SessionId,
    time::{DateTime, Duration},
};

const SIGNUP_SESSION: &str = "signup-session";
const SIGNIN_SESSION: &str = "signin-session";
const SESSION: &str = "session";
const PASSWORD: &str = "password";

/// Function related to user's table.
pub trait UserTable {
    /// Check if the user has been already registered.
    fn user_is_registered(&self, username: &str) -> Result<bool>;

    /// Register a new user, removing the used invitation.
    fn register_user_password(&self, username: &str, password_file: PasswordFile) -> Result<()>;
}

impl UserTable for KVStorage {
    fn user_is_registered(&self, username: &str) -> Result<bool> {
        self.read()?
            .has(format!("{PASSWORD}:{username}"))
            .map_err(Into::into)
    }

    fn register_user_password(&self, username: &str, password_file: PasswordFile) -> Result<()> {
        self.write()
            .set(format!("{PASSWORD}:{username}"), &password_file)
            .map_err(Into::into)
    }
}

/// Sign up session
#[derive(Deserialize, Serialize)]
pub struct SignupSession {
    pub username: String,
    created_at: DateTime,
}

impl SignupSession {
    const LIFETIME: Duration = Duration::minutes(1);

    /// Create a new signup session with the given data.
    pub fn new(username: String) -> Self {
        Self {
            username,
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
