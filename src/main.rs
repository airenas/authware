pub mod model;
use authware::auth::sample::Sample;
use authware::model::config::SessionConfig;
use authware::model::service;
use authware::store::encryptor::MagicEncryptor;
use authware::store::memory::InMemorySessionStore;
use authware::store::redis::RedisSessionStore;
use authware::tls::cert::generate_certificates;
use authware::utils::ip_extractor;
use authware::{handler, shutdown_signal, AuthService, Encryptor, IPExtractor, SessionStore};
use axum::http::HeaderName;
use axum_server::tls_rustls::RustlsConfig;
use deadpool_redis::{Config, Runtime};
use humantime::format_duration;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use axum::{
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
    /// Sample users list, format: user:pass;user:pass
    #[arg(long, env, default_value = "admin:admin;user:user")]
    sample_users: String,
    /// host for certificate generation    
    #[arg(long, env, default_value = "localhost")]
    host: String,
    /// redis url
    #[arg(long, env, default_value = "")]
    redis_url: String,
    // data encryption key
    #[arg(long, env, default_value = "", required = true)]
    encryption_key: String,
    // index of ip in x-forwarded-for header, -1 - the last, -2 - the one before the last
    #[arg(long, env, default_value = "-2")]
    ip_index: i16,

    // authentication ws url
    #[arg(long, env, default_value = "", required = false)]
    auth_ws_url: String,
    // authentication ws basic auth user
    #[arg(long, env, default_value = "", required = false)]
    auth_ws_user: String,
    // authentication ws basic auth pass
    #[arg(long, env, default_value = "", required = false)]
    auth_ws_pass: String,
    // app code in authentication ws
    #[arg(long, env, default_value = "", required = false)]
    auth_app_code: String,
}

async fn main_int(args: Args) -> anyhow::Result<()> {
    log::info!("Starting authware");
    tracing::info!(version = env!("CARGO_APP_VERSION"));
    tracing::info!(port = args.port, "cfg");

    tracing::info!(
        session_timeout = format_duration(args.session_timeout).to_string(),
        "cfg"
    );
    tracing::info!(
        inactivity_timeout = format_duration(args.inactivity_timeout).to_string(),
        "cfg"
    );
    tracing::info!(port = args.port, "port");

    let config = SessionConfig {
        inactivity: args.inactivity_timeout.as_millis() as i64,
        session_timeout: args.session_timeout.as_millis() as i64,
    };

    let store: Box<dyn SessionStore + Send + Sync> = if args.redis_url.is_empty() {
        log::warn!("Using in-memory store");
        Box::new(InMemorySessionStore::new())
    } else {
        log::info!("Using redis store");
        let cfg = Config::from_url(args.redis_url.clone());
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        let encryptor: Box<dyn Encryptor + Send + Sync> =
            Box::new(MagicEncryptor::new(&args.encryption_key)?);
        Box::new(RedisSessionStore::new(pool, encryptor))
    };

    let auth: Box<dyn AuthService + Send + Sync> = init_auth(&args).await?;

    let ip_extractor: Box<dyn IPExtractor + Send + Sync> =
        Box::new(ip_extractor::Header::new(args.ip_index));
    let service_data = service::Data {
        config,
        store,
        auth_service: auth,
        ip_extractor,
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
        .route("/auth/live", get(handler::live::handler))
        .route("/auth/login", post(handler::login::handler))
        .route("/auth/logout", post(handler::logout::handler))
        .route("/auth/keep-alive", post(handler::keep_alive::handler))
        .route("/auth", get(handler::auth::handler))
        .with_state(quarded_data)
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(15)),
            cors,
        ));

    let (cert, key_pair) = generate_certificates(&args.host)?;
    tracing::trace!("Configuring Rustls");
    let cfg = RustlsConfig::from_der(vec![cert.der().to_vec()], key_pair.serialize_der()).await?;
    tracing::debug!(port = args.port, "starting https");
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

    let handle = axum_server::Handle::new();
    let shutdown_future = shutdown_signal_handle(handle.clone());
    tokio::spawn(shutdown_future);

    tracing::info!(addr = format!("{}", addr), "listening");
    axum_server::bind_rustls(addr, cfg)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();

    tracing::info!("Bye");
    Ok(())
}

async fn init_auth(args: &Args) -> anyhow::Result<Box<dyn AuthService + Send + Sync>> {
    let mut auths: Vec<Box<dyn AuthService + Send + Sync>> = Vec::new();
    if !args.sample_users.is_empty() {
        tracing::warn!("Using sample auth");
        auths.push(Box::new(Sample::new(&args.sample_users)?));
    }
    if !args.auth_ws_url.is_empty() {
        tracing::info!("Using sample admin3ws auth");
        auths.push(Box::new(authware::auth::admin3ws::Auth::new(
            &args.auth_ws_url,
            &args.auth_ws_user,
            args.auth_ws_pass.as_str().into(),
            &args.auth_app_code,
        )?));
    }
    if auths.is_empty() {
        return Err(anyhow::anyhow!("No auth method specified"));
    }
    if auths.len() == 1 {
        return Ok(auths.pop().unwrap());
    }
    Ok(Box::new(authware::auth::combined::Auths::new(auths)?))
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
    tracing::trace!("Received termination signal shutting down");
    handle.graceful_shutdown(Some(Duration::from_secs(10)));
}
