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

use crate::{config::Config, opaque::OpaqueSignature, user};

use self::state::AppState;

mod session;
mod signin;
mod signout;
mod signup;
mod state;

/// Launch the management server listening on the given port
pub async fn serve(config: &Config) -> Result<()> {
    let storage = KVStorage::open(&config.storage)?;
    let signature = OpaqueSignature::new(&config.opaque_signature)?;
    let auth_layer = ValidateRequestHeaderLayer::bearer(&config.auth_token);

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

    let fresh_addr = (Ipv4Addr::LOCALHOST, 8000);
    let reverse_proxy = ReverseProxy::new(fresh_addr);

    let state = AppState::new(storage, signature);
    let router = Router::new()
        .route("/api/health", get(health))
        .route(
            "/api/session/:id",
            get(session::get_session).layer(auth_layer.clone()),
        )
        .route(
            "/api/signup/invitation/:code",
            get(signup::get_invitation).layer(auth_layer.clone()),
        )
        .route("/api/signup/start", post(signup::start))
        .route("/api/signup/finish", post(signup::finish))
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
