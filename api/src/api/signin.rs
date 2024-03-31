use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{opaque, session::SessionId, user};

use super::state::AppState;

#[derive(Deserialize)]
pub struct StartReq {
    username: String,
    message: opaque::LoginRequest,
}

#[derive(Serialize)]
pub struct StartRes {
    #[serde(serialize_with = "SessionId::serialize")]
    session: SessionId,
    message: opaque::LoginResponse,
}

/// First step of login.
pub async fn start(
    State(state): State<AppState>,
    Json(req): Json<StartReq>,
) -> Result<Json<StartRes>, StatusCode> {
    let StartReq {
        username,
        message: login_request,
    } = req;

    let password_file = user::get_password_file(state.storage(), &username).map_err(|err| {
        tracing::error!("failed to retrieve password file: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    if password_file.is_none() {
        tracing::info!("user {} not registered", username);
    }

    let (login_response, login_state) =
        opaque::login_start(state.signature(), &username, password_file, login_request).map_err(
            |err| {
                tracing::error!("failed to start login of user {username}: {err}",);
                StatusCode::INTERNAL_SERVER_ERROR
            },
        )?;

    let session = user::SigninSession::new(login_state);
    let session_id = user::push_signin_session(state.storage(), session).map_err(|err| {
        tracing::error!("failed to push signin session: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(StartRes {
        session: session_id,
        message: login_response,
    }))
}

#[derive(Deserialize)]
pub struct FinishReq {
    session: SessionId,
    message: opaque::LoginFinalization,
}

/// Finish login.
pub async fn finish(
    State(state): State<AppState>,
    Json(req): Json<FinishReq>,
) -> Result<(), StatusCode> {
    let FinishReq {
        session: session_id,
        message: login_finalization,
    } = req;

    let user::SigninSession {
        state: login_state, ..
    } = user::pull_signin_session(state.storage(), session_id)
        .map_err(|err| {
            tracing::error!("failed to retrieve signin session: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    opaque::login_finish(login_state, login_finalization).map_err(|err| {
        tracing::error!("login failed: {err}");
        StatusCode::UNAUTHORIZED
    })?;

    Ok(())
}
