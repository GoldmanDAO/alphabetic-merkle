use sea_orm:: {
  Database, 
  DatabaseConnection, 
  ConnectOptions,
  error::DbErr,
  RuntimeErr
};
use std::time::Duration;
use serde_json::json;

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
    //.set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

  let db = Database::connect(opt).await?;
  Ok(db)
}

fn get_sql_error(error: DbErr) -> sqlx::error::ErrorKind {
  match error {
    DbErr::Query(RuntimeErr::SqlxError(sql_error)) => match sql_error {
        sqlx::Error::Database(e) => {
          e.kind()
        }
        _ => panic!("Unexpected database error: {:?}", sql_error),
    },
    _ => panic!("Unexpected database error: {:?}", error)
  }
}



#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stdout)
        .init();

  let db = init_database().await.unwrap();

  let data: serde_json::Value = json!({
    "name": "Apple",
    "author": "0x97054e06ac17c7efaa7b10e9d6b7ed753fc3f26",
    "block_number": 1,
    "ipfs_hash": "http://example.com",
    "root_hash": "0x0",
  });

  let _prop = create_proposal(&db, data).await.unwrap();

  let proposals = get_proposals(&db).await.unwrap();
  
  println!("proposals: {:?}", proposals);
}