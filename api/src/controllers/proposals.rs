use axum::{extract::{State, Path}, Json, http::StatusCode, response::IntoResponse};
use sea_orm::{ActiveModelTrait, Set, EntityTrait, TransactionTrait, error::DbErr, DatabaseConnection};
use entity::prelude::*;
use serde::Deserialize;
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
    get_merkle_root,
    generate_proof_of_inclusion,
    generate_proof_of_absense, Error
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

#[derive(Deserialize)]
pub struct ProposalsWithAccountsActiveModel {
  proposal: Value,
  accounts: Vec<Value>
}

pub async fn create_proposal(
  state: State<AppState>,
  Json(payload): Json<ProposalsWithAccountsActiveModel>,
) -> (StatusCode, Json<Value>) {

  let res: Result<ProposalsModel, DbErr> = create_proposal_transaction(&state.conn, payload.proposal, payload.accounts).await;

  match res {
    Ok(r) => (StatusCode::CREATED, Json(json!(r))),
    Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))
  }
}

fn get_merkletree_root(accounts: Vec<AccountsActiveModel>) -> Result<String, Error> {
  let accounts: Vec<AccountWithBalance> = accounts.iter().map(|account| {
    let address = account.address.clone().take();
    let balance = account.balance.clone().take();
    match (address, balance) {
        (Some(address), Some(balance)) => AccountWithBalance::new(address.as_str(), balance.as_str()),
        _ => panic!("Error getting accounts") //TODO: handle error
    }
  }).collect();
  let merkle_root = get_merkle_root(&accounts)?;
  Ok(hex::encode(merkle_root))
}

async fn create_proposal_transaction(
  db: &DatabaseConnection,
  proposal_json: Value,
  accounts_json: Vec<Value>
) -> Result<ProposalsModel, DbErr> {
  let proposal_data = ProposalsActiveModel::from_json(proposal_json)?;

  let proposal_id = db.transaction::<_, Option<i32>, DbErr>(|txn| {
    Box::pin(async move {
      let mut proposal: ProposalsActiveModel = proposal_data
        .save(txn)
        .await?;

      let accounts: Vec<AccountsActiveModel> = accounts_json.iter().map(|account_json| {
        let mut account_model = AccountsActiveModel::from_json(account_json.clone()).unwrap();
        account_model.set(AccountsBase::Column::ProposalId,proposal.id.clone().into_value().unwrap());
        account_model
      }).collect();

      Accounts::insert_many(accounts.clone()).exec(txn).await?;

      let proposal_id = proposal.id.clone().take();

      let merkle_root = get_merkletree_root(accounts)
        .map_err(|e| DbErr::Custom(format!("Error creating merkle tree: {}", e)))?;
      proposal.root_hash = Set(merkle_root);
      proposal.update(txn).await?;

      Ok(proposal_id)
    })
  }).await
  .map_err(|e: sea_orm::TransactionError<DbErr>| DbErr::Custom(e.to_string()))?
  .ok_or(DbErr::Custom("Error creating proposal".to_string()))?;

  let proposals_with_accounts: ProposalsModel = get_proposal_with_accounts(db, proposal_id).await?;

  Ok(proposals_with_accounts)
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
