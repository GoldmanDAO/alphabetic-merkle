use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use entity::prelude::*;
use ethers::utils::hex;
use merkletree::{
    account_with_balance::AccountWithBalance,
    merkle_tree::{generate_proof_of_absense, generate_proof_of_inclusion},
};
use sea_orm::{error::DbErr, ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use services::accounts::queries::get_accounts_by_proposal_id;
use services::proposals::queries::{get_proposal_with_accounts, insert_proposal, list_proposals};
use services::utils::pagination::Pagination;

use crate::AppState;

pub async fn get_proposals(
    state: State<AppState>,
    payload: Option<Json<Pagination>>,
) -> impl IntoResponse {
    let pag = payload.map(|Json(payload)| payload);
    let proposals = list_proposals(&state.conn, pag).await;

    match proposals {
        Ok(proposals) => (StatusCode::OK, Json(json!(proposals))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("[Invalid payload] {}", e)})),
        ),
    }
}

pub async fn get_proposal(state: State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    let proposals_with_accounts = get_proposal_with_accounts(&state.conn, id).await;

    match proposals_with_accounts {
        Ok(proposal) => (StatusCode::OK, Json(json!(proposal))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("{}", e)})),
        ),
    }
}

#[derive(Deserialize, Serialize)]
pub struct NewAccount {
    pub address: String,
    pub balance: String,
}

#[derive(Deserialize, Serialize)]
pub struct NewProposal {
    pub author: String,
    pub block_number: i64,
    pub ipfs_hash: String,
    pub accounts: Vec<NewAccount>,
}

async fn insert_proposal_with_accounts(
    db: &DatabaseConnection,
    proposal_data: ProposalsActiveModel,
    accounts_data: Vec<AccountsActiveModel>,
) -> Result<ProposalsModel, DbErr> {
    let proposal_id = insert_proposal(db, proposal_data, accounts_data).await?;
    let proposals_with_accounts = get_proposal_with_accounts(db, proposal_id).await?;
    Ok(proposals_with_accounts)
}

pub async fn create_proposal(
    state: State<AppState>,
    Json(payload): Json<NewProposal>,
) -> (StatusCode, Json<Value>) {
    let proposal_data: ProposalsActiveModel =
        ProposalsActiveModel::from_json(json!(payload)).unwrap();
    let accounts_data: Vec<AccountsActiveModel> = payload
        .accounts
        .iter()
        .map(|account| AccountsActiveModel::from_json(json!(account)).unwrap())
        .collect();

    let proposals_with_accounts =
        insert_proposal_with_accounts(&state.conn, proposal_data, accounts_data).await;

    match proposals_with_accounts {
        Ok(r) => (StatusCode::CREATED, Json(json!(r))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("{}", e)})),
        ),
    }
}

pub async fn get_proof_of_inclusion(
    state: State<AppState>,
    Path(proposal_id): Path<i32>,
    Json(account): Json<AccountWithBalance>,
) -> (StatusCode, Json<Value>) {
    let accounts = get_accounts_by_proposal_id(&state.conn, proposal_id)
        .await
        .unwrap();
    let accounts_with_balance: Vec<AccountWithBalance> = accounts
        .iter()
        .map(|account| {
            AccountWithBalance::new(
                account.address.clone().as_str(),
                account.balance.clone().as_str(),
            )
        })
        .collect();

    let res = generate_proof_of_inclusion(&accounts_with_balance, account);

    match res {
        Ok(proof) => (StatusCode::OK, Json(json!({"proof": hex::encode(proof)}))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("{}", e)})),
        ),
    }
}

pub async fn get_proof_of_absense(
    state: State<AppState>,
    Path(proposal_id): Path<i32>,
    Json(account): Json<AccountWithBalance>,
) -> (StatusCode, Json<Value>) {
    let accounts = get_accounts_by_proposal_id(&state.conn, proposal_id)
        .await
        .unwrap();
    let accounts_with_balance: Vec<AccountWithBalance> = accounts
        .iter()
        .map(|account| {
            AccountWithBalance::new(
                account.address.clone().as_str(),
                account.balance.clone().as_str(),
            )
        })
        .collect();

    let res = generate_proof_of_absense(&accounts_with_balance, account);

    match res {
        Ok(proof) => {
            let hex_proof = match proof {
                (Some(left), Some(right)) => (hex::encode(left), hex::encode(right)),
                (Some(left), None) => (hex::encode(left), "".to_string()),
                (None, Some(right)) => ("".to_string(), hex::encode(right)),
                (None, None) => ("".to_string(), "".to_string()),
            };
            (StatusCode::OK, Json(json!({"proof": hex_proof })))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("{}", e)})),
        ),
    }
}

pub async fn download_accounts_csv(
    state: State<AppState>,
    Path(proposal_id): Path<i32>,
) -> impl IntoResponse {
    let accounts_csv: String = get_accounts_by_proposal_id(&state.conn, proposal_id)
        .await
        .map(|accounts| {
            accounts
                .iter()
                .map(|account| format!("{},{}", account.address, account.balance))
                .collect::<Vec<String>>()
                .join("\n")
        })
        .unwrap_or("".to_string());

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"accounts.csv\""),
    );

    (headers, accounts_csv)
}
