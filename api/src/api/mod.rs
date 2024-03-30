use anyhow::Result;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use mello::kvstorage::KVStorage;
use tokio::net::TcpListener;

use crate::{config::Config, opaque::OpaqueServer, user};

use self::state::AppState;

mod signup;
mod state;

/// Launch the management server listening on the given port
pub async fn serve(config: &Config) -> Result<()> {
    let storage = KVStorage::open(&config.storage)?;
    let opaque = OpaqueServer::new(&config.private_key);

    let first_invitation = user::create_first_signup_invitation(&storage, &config.admin_user)?;
    if let Some(invitation_code) = first_invitation {
        tracing::info!(
            "'{}' invitation code is '{}'",
            &config.admin_user,
            invitation_code.display()
        );
    }

    let listener = TcpListener::bind(&config.listen_addr).await?;
    let local_addr = listener.local_addr()?;
    tracing::info!("listening on {}", local_addr);

    let state = AppState::new(storage, opaque);
    let router = Router::new()
        .route("/api/health", get(health))
        .route("/api/signup/invitation/:code", get(signup::get_invitation))
        .route("/api/signup/start", post(signup::start))
        .route("/api/signup/finish", post(signup::finish))
        .with_state(state);

    axum::serve(listener, router).await?;
    Ok(())
}

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}
