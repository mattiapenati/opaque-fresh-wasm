use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    invitation::{Invitation, InvitationCode},
    opaque,
    session::SessionId,
    user,
};

use super::state::AppState;

/// Retrieve the signup invitation details from its code.
pub async fn get_invitation(
    State(state): State<AppState>,
    Path(code): Path<InvitationCode>,
) -> Result<Json<Invitation>, StatusCode> {
    user::get_signup_invitation(state.storage(), &code)
        .map_err(|err| {
            tracing::error!("failed to retrieve invitation from database: {err}",);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
pub struct StartReq {
    code: InvitationCode,
    username: String,
    message: opaque::RegistrationRequest,
}

#[derive(Serialize)]
pub struct StartRes {
    #[serde(serialize_with = "SessionId::serialize")]
    session: SessionId,
    message: opaque::RegistrationResponse,
}

/// First step of registration.
pub async fn start(
    State(state): State<AppState>,
    Json(req): Json<StartReq>,
) -> Result<Json<StartRes>, StatusCode> {
    let StartReq {
        code,
        username,
        message: registration_request,
    } = req;

    let signup_invitation_is_valid =
        user::signup_invitation_is_valid(state.storage(), &code, &username).map_err(|err| {
            tracing::error!("failed to check signup invitation: {err}",);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    if !signup_invitation_is_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let registration_response =
        opaque::registration_start(state.signature(), &username, registration_request).map_err(
            |err| {
                tracing::error!("failed to start registration of user {username}: {err}",);
                StatusCode::INTERNAL_SERVER_ERROR
            },
        )?;

    let session = user::SignupSession::new(code);
    let session_id = user::push_signup_session(state.storage(), session).map_err(|err| {
        tracing::error!("failed to push signup session: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(StartRes {
        session: session_id,
        message: registration_response,
    }))
}

#[derive(Deserialize)]
pub struct FinishReq {
    session: SessionId,
    message: opaque::RegistrationUpload,
}

/// Finish registration.
pub async fn finish(
    State(state): State<AppState>,
    Json(req): Json<FinishReq>,
) -> Result<Json<()>, StatusCode> {
    let FinishReq {
        session: session_id,
        message: registration_upload,
    } = req;

    let user::SignupSession { code, .. } = user::pull_signup_session(state.storage(), session_id)
        .map_err(|err| {
            tracing::error!("failed to retrieve signup session: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let password_file = opaque::registration_finish(registration_upload);
    user::register_user_password(state.storage(), &code, password_file).map_err(|err| {
        tracing::error!("failed to save user's password file: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(()))
}
