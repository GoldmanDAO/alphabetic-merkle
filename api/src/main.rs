use anyhow::Ok;
use axum::{
  routing::{get, post},
  Router, Server,
};
use sea_orm:: {
  Database, 
  DatabaseConnection, 
  ConnectOptions,
};

use tower::ServiceBuilder;
use tower_http::{
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
    ServiceBuilderExt,
};

use tracing_subscriber::{Registry, prelude::__tracing_subscriber_SubscriberExt, Layer, filter::LevelFilter};
use std::{str::FromStr, time::Duration};
use std::{env, net::SocketAddr};

mod controllers;
use controllers::proposals::{
    get_proposals, 
    create_proposal, 
    get_proof_of_inclusion, 
    get_proof_of_absense
};

use crate::controllers::proposals::get_proposal;

async fn init_database() ->  anyhow::Result<DatabaseConnection> {
  let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

  let mut opt = ConnectOptions::new(db_url); //TODO Use env var
  opt.max_connections(100)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(8))
    .acquire_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8))
    .max_lifetime(Duration::from_secs(8))
    .sqlx_logging(true);
    //.sqlx_logging_level(log::LevelFilter::Info)

  let db = Database::connect(opt).await?;
  Ok(db)
}

async fn init_tracing() {
  let stdout_log = tracing_subscriber::fmt::Layer::new()
    .with_writer(std::io::stdout)
    .pretty()
    .with_filter(LevelFilter::DEBUG);

  let subscriber = Registry::default()
    .with(stdout_log);

  tracing::subscriber::set_global_default(subscriber)
    .expect("setting tracing failed");
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
      .route("/proposal/:id/inclusion_proof", post(get_proof_of_inclusion))
      .route("/proposal/:id/absense_proof", post(get_proof_of_absense))
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
  // Migrator::up(&conn, None).await.unwrap();

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
