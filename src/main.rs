pub mod model;
use authware::auth::sample::Sample;
use authware::model::config::SessionConfig;
use authware::model::service;
use authware::store::memory::InMemorySessionStore;
use authware::tls::cert::generate_certificates;
use authware::{handler, shutdown_signal, AuthService, SessionStore};
use axum::http::HeaderName;
use axum_server::tls_rustls::RustlsConfig;
use humantime::format_duration;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Sound saver http service
#[derive(Parser, Debug)]
#[command(version = env!("CARGO_APP_VERSION"), name = "authware", about, long_about = None)]
struct Args {
    /// Server port
    #[arg(long, env, default_value = "8000")]
    port: u16,
    /// Session timeout
    #[arg(long, env, default_value = "6h", value_parser = humantime::parse_duration)]
    session_timeout: Duration,
    /// Inactivity timeout
    #[arg(long, env, default_value = "30m", value_parser = humantime::parse_duration)]
    inactivity_timeout: Duration,
    /// Sample user
    #[arg(long, env, default_value = "admin")]
    sample_user: String,
    /// Sample user
    #[arg(long, env, default_value = "")]
    sample_user_pass: String,
    #[arg(long, env, default_value = "localhost")]
    host: String,
}

async fn main_int(args: Args) -> anyhow::Result<()> {
    log::info!("Starting authware");
    tracing::info!(version = env!("CARGO_APP_VERSION"));
    tracing::info!(port = args.port, "port");

    tracing::info!(
        session_timeout = format_duration(args.session_timeout).to_string(),
        "cfg"
    );
    tracing::info!(
        inactivity_timeout = format_duration(args.inactivity_timeout).to_string(),
        "cfg"
    );
    tracing::info!(port = args.port, "port");
    log::debug!("Init tracing...");

    let config = SessionConfig {
        inactivity: args.inactivity_timeout.as_millis() as i64,
        session_timeout: args.session_timeout.as_millis() as i64,
    };
    let store: Box<dyn SessionStore + Send + Sync> = Box::new(InMemorySessionStore::new());
    let sample_auth: Box<dyn AuthService + Send + Sync> =
        Box::new(Sample::new(&args.sample_user, &args.sample_user_pass)?);
    let service_data = service::Data {
        config,
        store,
        auth_service: sample_auth,
    };
    let quarded_data = Arc::new(service_data);

    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_origin(Any)
        .allow_headers(vec![
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
        ]);

    let app = Router::new()
        .route("/live", get(handler::live::handler))
        .route("/auth/login", post(handler::login::handler))
        .route("/auth/logout", post(handler::logout::handler))
        .route("/auth/keep-alive", post(handler::keep_alive::handler))
        .route("/auth", get(handler::auth::handler))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(500 * 1024 * 1024))
        .with_state(quarded_data)
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(40)),
            cors,
        ));

    let (cert, key_pair) = generate_certificates(&args.host)?;
    tracing::info!("Configuring Rustls");
    let cfg = RustlsConfig::from_der(vec![cert.der().to_vec()], key_pair.serialize_der()).await?;
    tracing::info!(port = args.port, "starting https");
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

    let handle = axum_server::Handle::new();
    let shutdown_future = shutdown_signal_handle(handle.clone());
    tokio::spawn(shutdown_future);

    tracing::debug!("listening on {}", addr);
    axum_server::bind_rustls(addr, cfg)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();

    tracing::info!("Bye");
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();
    let args = Args::parse();
    if let Err(e) = main_int(args).await {
        log::error!("{}", e);
        return Err(e);
    }
    Ok(())
}

async fn shutdown_signal_handle(handle: axum_server::Handle) {
    shutdown_signal().await;
    tracing::info!("Received termination signal shutting down");
    handle.graceful_shutdown(Some(Duration::from_secs(10)));
}
