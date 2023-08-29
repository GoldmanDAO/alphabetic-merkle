
use axum::{extract::State, Json, http::StatusCode, response::IntoResponse};
use sea_orm::{ActiveModelTrait, TryIntoModel};
use entity::prelude::*;
use serde_json::{Value, json};
use services::proposals::queries:: {
  insert_proposal,
  list_proposals,
};
use services::proposals::pagination::Pagination;

use crate::AppState;

pub async fn get_proposals(
  state: State<AppState>,
  payload: Option<Json<Pagination>>,
) -> impl IntoResponse {
  let pag = payload.map(|Json(payload)| payload);
  let proposals = list_proposals(&state.conn, pag).await;

  match proposals {
    Ok(proposals) => (StatusCode::OK, Json(json!(proposals))),
    Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("[Invalid payload] {}", e)})))
  }
}

pub async fn create_proposal(
  state: State<AppState>,
  Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
  let proposal_data = ProposalsActiveModel::from_json(payload);
  match proposal_data {
    Ok(proposal) => {
      let proposal = insert_proposal(&state.conn, proposal).await;
      match proposal {
        Ok(proposal) => (StatusCode::CREATED, Json(json!(proposal.try_into_model().unwrap()))),
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("[Invalid payload] {}", e)})))
      }
    }
    Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("[Invalid payload] {}", e)})))
  }
}