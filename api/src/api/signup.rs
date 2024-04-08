use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    invitation::{Invitation, InvitationCode},
    opaque,
    session::SessionId,
    user::{self, UserTable},
};

use super::state::AppState;

#[derive(Deserialize)]
#[serde(tag = "step")]
pub enum Request {
    Start(StartReq),
    Finish(FinishReq),
}

#[derive(Serialize)]
#[serde(untagged)]
enum Response {
    Start(StartRes),
    Finish(FinishRes),
}

/// Registration endpoint
pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<Request>,
) -> Result<impl IntoResponse, StatusCode> {
    let res = match req {
        Request::Start(req) => Response::Start(start(state, req).await?),
        Request::Finish(req) => Response::Finish(finish(state, req).await?),
    };
    Ok(Json(res))
}

#[derive(Deserialize)]
pub struct StartReq {
    code: InvitationCode,
    message: opaque::RegistrationRequest,
}

#[derive(Serialize)]
struct StartRes {
    #[serde(serialize_with = "SessionId::serialize")]
    session: SessionId,
    message: opaque::RegistrationResponse,
}

/// First step of registration.
async fn start(state: AppState, req: StartReq) -> Result<StartRes, StatusCode> {
    let StartReq {
        code,
        message: registration_request,
    } = req;

    let Invitation { username, .. } = state
        .invitation_key()
        .verify(&code)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let registration_response =
        opaque::registration_start(state.signature(), &username, registration_request).map_err(
            |err| {
                tracing::error!("failed to start registration of user {username}: {err}",);
                StatusCode::INTERNAL_SERVER_ERROR
            },
        )?;

    let session = user::SignupSession::new(username);
    let session_id = user::push_signup_session(state.storage(), session).map_err(|err| {
        tracing::error!("failed to push signup session: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StartRes {
        session: session_id,
        message: registration_response,
    })
}

#[derive(Deserialize)]
pub struct FinishReq {
    session: SessionId,
    message: opaque::RegistrationUpload,
}

#[derive(Serialize)]
struct FinishRes {}

/// Finish registration.
async fn finish(state: AppState, req: FinishReq) -> Result<FinishRes, StatusCode> {
    let FinishReq {
        session: session_id,
        message: registration_upload,
    } = req;

    let user::SignupSession { username, .. } =
        user::pull_signup_session(state.storage(), session_id)
            .map_err(|err| {
                tracing::error!("failed to retrieve signup session: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::UNAUTHORIZED)?;

    let password_file = opaque::registration_finish(registration_upload);
    state
        .storage()
        .register_user_password(&username, password_file)
        .map_err(|err| {
            tracing::error!("failed to save user's password file: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(FinishRes {})
}
