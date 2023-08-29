use anyhow::Ok;
use axum::{
  routing::get,
  Router, Server,
};
use sea_orm:: {
  Database, 
  DatabaseConnection, 
  ConnectOptions,
};

use tracing_subscriber::{Registry, prelude::__tracing_subscriber_SubscriberExt, Layer, filter::LevelFilter};
use std::{str::FromStr, time::Duration};
use std::{env, net::SocketAddr};

mod controllers;
use controllers::proposals::{get_proposals, create_proposal};

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

    let app = Router::new()
      .route("/", get(get_proposals).post(create_proposal))
      //.route("/:id", get(edit_post).post(update_post))
      //.route("/new", get(new_post))
      //.route("/delete/:id", post(delete_post))
      .with_state(state);

    let addr = SocketAddr::from_str(&server_url).unwrap();
    Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

#[tokio::main]
async fn start() -> anyhow::Result<()> {
  env::set_var("RUST_LOG", "debug");
  
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

/*
#[derive(Deserialize)]
struct Params {
    page: Option<u64>,
    posts_per_page: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}

async fn list_posts(
    state: State<AppState>,
    Query(params): Query<Params>,
    cookies: Cookies,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(5);

    let (posts, num_pages) = QueryCore::find_posts_in_page(&state.conn, page, posts_per_page)
        .await
        .expect("Cannot find posts in page");

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie::<FlashData>(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = state
        .templates
        .render("index.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn new_post(state: State<AppState>) -> Result<Html<String>, (StatusCode, &'static str)> {
    let ctx = tera::Context::new();
    let body = state
        .templates
        .render("new.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn create_post(
    state: State<AppState>,
    mut cookies: Cookies,
    form: Form<post::Model>,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    let form = form.0;

    MutationCore::create_post(&state.conn, form)
        .await
        .expect("could not insert post");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Post succcessfully added".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

async fn edit_post(
    state: State<AppState>,
    Path(id): Path<i32>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let post: post::Model = QueryCore::find_post_by_id(&state.conn, id)
        .await
        .expect("could not find post")
        .unwrap_or_else(|| panic!("could not find post with id {id}"));

    let mut ctx = tera::Context::new();
    ctx.insert("post", &post);

    let body = state
        .templates
        .render("edit.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}

async fn update_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
    form: Form<post::Model>,
) -> Result<PostResponse, (StatusCode, String)> {
    let form = form.0;

    MutationCore::update_post_by_id(&state.conn, id, form)
        .await
        .expect("could not edit post");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Post succcessfully updated".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}

async fn delete_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    mut cookies: Cookies,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    MutationCore::delete_post(&state.conn, id)
        .await
        .expect("could not delete post");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Post succcessfully deleted".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}
*/

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
