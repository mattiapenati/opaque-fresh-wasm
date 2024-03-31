use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::{session::SessionId, user};

use super::state::AppState;

#[derive(Serialize)]
pub struct Session {
    username: String,
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<SessionId>,
) -> Result<Json<Session>, StatusCode> {
    let session = user::get_session(state.storage(), session_id).map_err(|err| {
        tracing::error!("failed to search session: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let Some(session) = session else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    Ok(Json(Session {
        username: session.username,
    }))
}
