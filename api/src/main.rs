use anyhow::{Ok, Error};
use axum::{
    routing::{get, post},
    Router, Server, response::Response,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use tower::ServiceBuilder;
use tower_http::{
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit, ServiceBuilderExt,
};

use std::{env, net::SocketAddr};
use std::{str::FromStr, time::Duration};
use tracing_subscriber::{
    filter::LevelFilter, prelude::__tracing_subscriber_SubscriberExt, Layer, Registry,
};

mod controllers;
use controllers::proposals::{
    create_proposal, get_proof_of_absense, get_proof_of_inclusion, get_proposals, download_accounts_csv,
};

use crate::controllers::proposals::get_proposal;

async fn init_database() -> anyhow::Result<DatabaseConnection> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut opt = ConnectOptions::new(db_url); //TODO Use env var
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    let db = Database::connect(opt).await?;
    Migrator::up(&db, None).await.unwrap();

    Ok(db)
}

async fn init_tracing() {
    let stdout_log = tracing_subscriber::fmt::Layer::new()
        .with_writer(std::io::stdout)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    let subscriber = Registry::default().with(stdout_log);

    tracing::subscriber::set_global_default(subscriber).expect("setting tracing failed");
}

async fn init_server(conn: DatabaseConnection) -> anyhow::Result<()> {
    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let server_url = format!("{host}:{port}");

    let state = AppState { conn };
    use axum::body::Bytes;
    // Build our middleware stack
    let middleware = ServiceBuilder::new()
        // Add high level tracing/logging to all requests
        .layer(
            TraceLayer::new_for_http()
                .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                    tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
                })
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
        )
        // Set a timeout
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        // Box the response body so it implements `Default` which is required by axum
        .map_response_body(axum::body::boxed)
        // Compress responses
        .compression();

    let app = Router::new()
        .route("/proposal", get(get_proposals).post(create_proposal))
        .route("/proposal/:id", get(get_proposal))
        .route(
            "/proposal/:id/inclusion_proof",
            post(get_proof_of_inclusion),
        )
        .route("/proposal/:id/absense_proof", post(get_proof_of_absense))
        .route("/proposal/:id/csv", get(download_accounts_csv))
        .layer(middleware)
        .with_state(state);

    let addr = SocketAddr::from_str(&server_url).unwrap();
    Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    init_tracing().await;

    let conn = init_database().await?;

    init_server(conn).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
