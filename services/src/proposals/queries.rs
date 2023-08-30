use sea_orm:: {
  DatabaseConnection, 
  error::DbErr,
  EntityTrait,
  ActiveModelTrait, 
  RuntimeErr,
  QueryOrder,
  PaginatorTrait,
  TransactionTrait, 
  Set,
};
use ethers::utils::hex;

use entity::prelude::*;
use crate::utils::pagination::Pagination;
use merkletree::{
  merkle_tree::{
    get_merkle_root,
    Error
  }, 
  account_with_balance::AccountWithBalance
};

async fn list_proposals_paginated(db: &DatabaseConnection, pag: Pagination) -> Result<Vec<ProposalsModel>, DbErr> {
  if !pag.check_range() {
    return Err(DbErr::Type("Invalid pagination range [1..100]".to_string()))
  }
  let paginated_proposals = Proposals::find()
  .order_by_asc(entity::proposals::Column::CreatedAt)
  .paginate(db, pag.per_page);

paginated_proposals.fetch_page(pag.page).await
}

pub async fn list_proposals(db: &DatabaseConnection, pag: Option<Pagination>) -> Result<Vec<ProposalsModel>, DbErr> {
  list_proposals_paginated(db, pag.unwrap_or_default()).await
}

pub async fn get_proposals_by_id(db: &DatabaseConnection, id: i32) -> Result<ProposalsModel, DbErr> {
  let proposal_res = Proposals::find_by_id(id).one(db).await;
  match proposal_res {
    Ok(proposal_opt) => match proposal_opt {
      Some(proposal) => Ok(proposal),
      None => Err(DbErr::Query(RuntimeErr::SqlxError(sqlx::error::Error::RowNotFound))),
    }
    Err(error) => Err(error),
  }
}

pub async fn get_proposal_with_accounts(db: &DatabaseConnection, id: i32) -> Result<ProposalsModel, DbErr> {
  let proposal_res = Proposals::find_by_id(id)
    .find_with_related(Accounts)
    .all(db).await?;

  proposal_res.first().ok_or(DbErr::Query(RuntimeErr::SqlxError(sqlx::error::Error::RowNotFound)))
    .map(|(proposal, accounts)| {
      let mut proposal = proposal.clone();
      proposal.accounts = accounts.clone();
      proposal
    })
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

pub async fn insert_proposal(db: &DatabaseConnection, mut proposal_data: ProposalsActiveModel, account_models: Vec<AccountsActiveModel>) -> Result<i32, DbErr> {

  let proposal_id = db.transaction::<_, Option<i32>, DbErr>(|txn| {
    Box::pin(async move {
      let mut accounts = account_models.clone();

      let merkle_root = get_merkletree_root(accounts.clone())
        .map_err(|e| DbErr::Custom(format!("Error creating merkle tree: {}", e)))?;
      proposal_data.root_hash = Set(merkle_root);

      let proposal: ProposalsActiveModel = proposal_data
        .save(txn)
        .await?;

      accounts.iter_mut().for_each(|account_model| {
        account_model.set(AccountsBase::Column::ProposalId,proposal.id.clone().into_value().unwrap());
      });

      Accounts::insert_many(accounts.clone()).exec(txn).await?;

      let proposal_id = proposal.id.clone().take();

      Ok(proposal_id)
    })
  }).await
  .map_err(|e: sea_orm::TransactionError<DbErr>| DbErr::Custom(e.to_string()))?
  .ok_or(DbErr::Custom("Error creating proposal".to_string()))?;

  Ok(proposal_id)
}