use axum::{extract::{State, Path}, Json, http::StatusCode, response::IntoResponse};
use sea_orm::{ActiveModelTrait, TryIntoModel, Set, EntityTrait};
use entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ethers::utils::hex;
use services::proposals::queries:: {
  insert_proposal,
  list_proposals,
  get_proposals_by_id,
};
use services::accounts::queries::get_accounts_by_proposal_id;
use services::utils::pagination::Pagination;
use merkletree::{merkle_tree::get_merkle_root, account_with_balance::AccountWithBalance};

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
  let proposal = get_proposals_by_id(&state.conn, id).await;
  
  let proposal_with_accounts = match proposal {
    Ok(proposal) => {
      let accounts = get_accounts_by_proposal_id(&state.conn, id).await;
      let accounts = match accounts {
        Ok(accounts) => accounts,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))
      };
      let proposal = proposal.try_into_model().unwrap();

      #[derive(Serialize)]
      struct ProposalWithAccounts {
        proposal: ProposalsModel,
        accounts: Vec<AccountsModel>
      };

      let proposal_with_accounts = ProposalWithAccounts {
        proposal,
        accounts
      };
      proposal_with_accounts
    },
    Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))
  };

  (StatusCode::OK, Json(json!(proposal_with_accounts)))
}

#[derive(Deserialize)]
struct ProposalsWithAccountsActiveModel {
  proposal: Value,
  accounts: Vec<Value>
}

pub async fn create_proposal(
  state: State<AppState>,
  Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
  let req: ProposalsWithAccountsActiveModel = serde_json::from_value(payload.clone()).unwrap();
  let proposal_data = ProposalsActiveModel::from_json(req.proposal);
  
  match proposal_data {
    Ok(proposal) => {
      let proposal: Result<ProposalsActiveModel, sea_orm::DbErr> = insert_proposal(&state.conn, proposal).await;
      let proposal: Result<(), sea_orm::DbErr> = match proposal {
        Ok(proposal) => {
          let accounts: Vec<AccountsActiveModel> = req.accounts.iter()
          .map(|account| {
            let mut account_model = AccountsActiveModel::from_json(account.clone()).unwrap();
            account_model.set(AccountsBase::Column::ProposalId,proposal.id.clone().into_value().unwrap());
            account_model
          })
          .collect();

          let res = Accounts::insert_many(accounts).exec(&state.conn).await;
          match res {
            Ok(r) => {
              let proposal = proposal.try_into_model().unwrap();

              let accounts: Vec<AccountsModel> = get_accounts_by_proposal_id(&state.conn, proposal.id.clone()).await.unwrap();
              let accounts: Vec<AccountWithBalance> = accounts.iter().map(|account| {
                AccountWithBalance::new(account.address.clone().as_str(), account.balance.clone().as_str())
              }).collect();
              let merkle_root = get_merkle_root(&accounts).unwrap();
              let merkle_root = hex::encode(merkle_root);
              let mut proposal: ProposalsActiveModel = proposal.into();
              proposal.root_hash = Set(merkle_root.clone());
              let proposal = proposal.update(&state.conn).await;
            }
            Err(e) => ()
          }
          Ok(())
        },
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))
      };
      
      (StatusCode::CREATED, Json(json!("{}")))
    }
    Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": format!("[Invalid payload] {}", e)})))
  }
}

//async fn get_proof_of_inclusion()
//async fn get_proof_of_absense()