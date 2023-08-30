use axum::{extract::{State, Path}, Json, http::StatusCode, response::IntoResponse};
use sea_orm::{ActiveModelTrait, error::DbErr, DatabaseConnection};
use entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ethers::utils::hex;
use services::proposals::queries:: {
  insert_proposal,
  list_proposals,
  get_proposal_with_accounts,
};
use services::accounts::queries::get_accounts_by_proposal_id;
use services::utils::pagination::Pagination;
use merkletree::{
  merkle_tree::{
    generate_proof_of_inclusion,
    generate_proof_of_absense,
  },
  account_with_balance::AccountWithBalance
};

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

pub async fn get_proposal(
  state: State<AppState>,
  Path(id): Path<i32>,
) -> impl IntoResponse {
  let proposals_with_accounts = get_proposal_with_accounts(&state.conn, id).await;

  match proposals_with_accounts {
    Ok(proposal) => (StatusCode::OK, Json(json!(proposal))),
    Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))
  }
}

#[derive(Deserialize, Serialize)]
pub struct NewAccount {
  pub address: String,
  pub balance: String
}

#[derive(Deserialize, Serialize)]
pub struct NewProposal {
  pub author: String,
  pub block_number: i64,
  pub ipfs_hash: String,
  pub accounts: Vec<NewAccount>
}

async fn insert_proposal_with_accounts(db: &DatabaseConnection, proposal_data: ProposalsActiveModel, accounts_data: Vec<AccountsActiveModel>) -> Result<ProposalsModel, DbErr> {
  let proposal_id = insert_proposal(db, proposal_data, accounts_data).await?;
  let proposals_with_accounts = get_proposal_with_accounts(db, proposal_id).await?;
  Ok(proposals_with_accounts)
}

pub async fn create_proposal(
  state: State<AppState>,
  Json(payload): Json<NewProposal>,
) -> (StatusCode, Json<Value>) {
  let proposal_data: ProposalsActiveModel = ProposalsActiveModel::from_json(json!(payload)).unwrap();
  let accounts_data: Vec<AccountsActiveModel> = payload.accounts.iter().map(|account| {
    AccountsActiveModel::from_json(json!(account)).unwrap()
  }).collect();

  let proposals_with_accounts = insert_proposal_with_accounts(&state.conn, proposal_data, accounts_data).await;

  match proposals_with_accounts {
    Ok(r) => (StatusCode::CREATED, Json(json!(r))),
    Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))
  }
}

pub async fn get_proof_of_inclusion(
  state: State<AppState>,
  Path(proposal_id): Path<i32>,
  Json(account): Json<AccountWithBalance>,
) -> (StatusCode, Json<Value>) {
  let accounts = get_accounts_by_proposal_id(&state.conn, proposal_id).await.unwrap();
  let accounts_with_balance: Vec<AccountWithBalance> = accounts.iter().map(|account| {
    AccountWithBalance::new(account.address.clone().as_str(), account.balance.clone().as_str())
  }).collect();

  let res = generate_proof_of_inclusion(&accounts_with_balance, account);

  match res {
      Ok(proof) => (StatusCode::OK, Json(json!({"proof": hex::encode(proof)}))),
      Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))
  }
}

pub async fn get_proof_of_absense(
  state: State<AppState>,
  Path(proposal_id): Path<i32>,
  Json(account): Json<AccountWithBalance>,
) -> (StatusCode, Json<Value>) {
  let accounts = get_accounts_by_proposal_id(&state.conn, proposal_id).await.unwrap();
  let accounts_with_balance: Vec<AccountWithBalance> = accounts.iter().map(|account| {
    AccountWithBalance::new(account.address.clone().as_str(), account.balance.clone().as_str())
  }).collect();

  let res = generate_proof_of_absense(&accounts_with_balance, account);

  match res {
    Ok(proof) => {
      let hex_proof = match proof {
        (Some(left), Some(right)) => (hex::encode(left), hex::encode(right)),
        (Some(left), None) => (hex::encode(left), "".to_string()),
        (None, Some(right)) => ("".to_string(), hex::encode(right)),
        (None, None) => ("".to_string(), "".to_string())
      };
      (StatusCode::OK, Json(json!({"proof": hex_proof })))
    },
    Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))
  }
}
