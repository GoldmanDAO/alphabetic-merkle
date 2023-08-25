
use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use sea_orm::error::DbErr;
use std::time::Duration;

use tracing_subscriber::{filter, Layer, layer::SubscriberExt, Registry};

pub fn init_logging() {
  let stdout_log = tracing_subscriber::fmt::Layer::new()
    .with_writer(std::io::stdout)
    .pretty()
    .with_filter(filter::LevelFilter::INFO);

  let subscriber = Registry::default()
    .with(stdout_log);

  tracing::subscriber::set_global_default(subscriber)
    .expect("setting tracing failed");
}

async fn init_database() -> Result<DatabaseConnection, DbErr> {
  let mut opt = ConnectOptions::new("postgres://postgres:postgres@localhost/test"); //TODO Use env var
  opt.max_connections(100)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(8))
    .acquire_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8))
    .max_lifetime(Duration::from_secs(8))
    .sqlx_logging(true);
    //.sqlx_logging_level(log::LevelFilter::Info)
    //.set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

  let db = Database::connect(opt).await?;
  Ok(db)
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stdout)
        .init();

  let db = init_database().await.unwrap();
  let res = db.ping().await.unwrap();
  println!("Hello, world!");
  res
}