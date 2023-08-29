
use entity::prelude::*;
use sea_orm:: {
  DatabaseConnection, 
  error::DbErr,
  EntityTrait,
  ActiveModelTrait, 
  RuntimeErr,
  QueryOrder,
  PaginatorTrait,
};

use super::pagination::Pagination;

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