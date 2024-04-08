use std::net::Ipv4Addr;

use anyhow::Result;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use mello::{kvstorage::KVStorage, reverse_proxy::ReverseProxy};
use tokio::net::TcpListener;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tower_otel::trace::HttpLayer;
use tracing::Level;

use crate::{
    config::Config,
    invitation::{Invitation, InvitationKey},
    opaque::OpaqueSignature,
    user::UserTable,
};

use self::state::AppState;

mod session;
mod signin;
mod signout;
mod signup;
mod state;

/// Launch the management server listening on the given port
pub async fn serve(config: &Config) -> Result<()> {
    let storage = KVStorage::open(&config.storage)?;
    let invitation_key: InvitationKey = config.key.invitation.parse()?;
    let signature = OpaqueSignature::new(&config.key.opaque)?;
    let auth_layer = ValidateRequestHeaderLayer::bearer(&config.key.session);

    // generate an invitation code for the administrator
    if !storage.user_is_registered(&config.admin)? {
        let username = &config.admin;
        let invitation = Invitation::admin(&username);
        let invitation_code = invitation_key.sign(&invitation);
        tracing::info!("'{username}' invitation code is '{invitation_code}'");
    }

    let listener = TcpListener::bind(&config.listen).await?;
    let local_addr = listener.local_addr()?;
    tracing::info!("listening on {}", local_addr);

    let fresh_addr = (Ipv4Addr::LOCALHOST, 8000);
    let reverse_proxy = ReverseProxy::new(fresh_addr);

    let signup = post(signup::signup).get_service(reverse_proxy.clone());

    let state = AppState::new(storage, signature, invitation_key);
    let router = Router::new()
        .route("/api/health", get(health))
        .route(
            "/api/session/:id",
            get(session::get_session).layer(auth_layer.clone()),
        )
        .route("/signup", signup)
        .route("/api/signin/start", post(signin::start))
        .route("/api/signin/finish", post(signin::finish))
        .route("/api/signout", get(signout::signout))
        .fallback_service(reverse_proxy)
        .with_state(state)
        .layer(HttpLayer::server(Level::INFO));

    axum::serve(listener, router).await?;
    Ok(())
}

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}
