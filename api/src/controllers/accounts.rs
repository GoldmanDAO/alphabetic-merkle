use axum::{extract::{State, Path}, Json, http::StatusCode, response::IntoResponse};
use sea_orm::{ActiveModelTrait, TryIntoModel};
use entity::prelude::*;
use serde_json::{Value, json};

use services::utils::pagination::Pagination;

use crate::AppState;

pub async fn get_accounts_by_proposal_id(
  state: State<AppState>,
  Path(id): Path<i32>,
) -> impl IntoResponse {
  
}
