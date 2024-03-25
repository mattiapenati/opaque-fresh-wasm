use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use tokio::net::TcpListener;

use crate::{
    config::Config,
    db::Database,
    invitation::{Invitation, InvitationCode},
};

use self::state::AppState;

mod state;

/// Create a new router for the mangement api
fn create_router(db: Database) -> Router {
    let state = AppState::new(db);
    Router::new()
        .route("/api/health", get(health))
        .route("/api/invitation/:code", get(get_signup_invitation))
        .with_state(state)
}

/// Launch the management server listening on the given port
pub async fn serve(config: &Config) -> Result<()> {
    let db = Database::new(&config.storage)?;
    let first_invitation = db.create_first_signup_invitation(&config.admin_user)?;
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

    let router = create_router(db);
    axum::serve(listener, router).await?;
    Ok(())
}

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn get_signup_invitation(
    State(state): State<AppState>,
    Path(code): Path<InvitationCode>,
) -> Result<Json<Invitation>, StatusCode> {
    state
        .get_signup_invitation(&code)
        .map_err(|err| {
            tracing::error!("failed to get signup invitation: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
