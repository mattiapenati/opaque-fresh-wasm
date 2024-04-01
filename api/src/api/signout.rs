use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;

use crate::user;

use super::state::AppState;

pub async fn signout(
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let Some(cookie) = jar.get(user::Session::COOKIE) else {
        return Err(StatusCode::OK);
    };
    let session_id = match cookie.value().parse() {
        Ok(session_id) => session_id,
        Err(err) => {
            tracing::error!("invalid session cookie: {err}");
            return Err(StatusCode::OK);
        }
    };

    let cookie = user::finish_session(state.storage(), session_id).map_err(|err| {
        tracing::error!("failed to remove session: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let jar = jar.add(cookie);
    Ok((jar, Json(())))
}
