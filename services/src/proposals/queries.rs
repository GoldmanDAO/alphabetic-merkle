use sea_orm:: {
  DatabaseConnection, 
  error::DbErr,
  EntityTrait,
  ActiveModelTrait, 
  RuntimeErr,
  QueryOrder,
  PaginatorTrait,
};

use entity::prelude::*;
use crate::utils::pagination::Pagination;
use crate::utils::errors::get_sql_error;

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

pub async fn insert_proposal(db: &DatabaseConnection, proposal_data: ProposalsActiveModel) -> Result<ProposalsActiveModel, DbErr> {
  match proposal_data.save(db).await {
    Ok(proposal) => Ok(proposal),
    Err(error) => {
      match get_sql_error(error) {
        sqlx::error::ErrorKind::CheckViolation => Err(DbErr::Type("Invalid author address".to_string())),
        _ => Err(DbErr::Custom("Error creating proposal".to_string()))
      }
    },
  }
}