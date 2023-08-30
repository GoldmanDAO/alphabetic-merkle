use sea_orm::{error::DbErr, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use entity::prelude::*;

pub async fn get_accounts_by_proposal_id(
    db: &DatabaseConnection,
    proposal_id: i32,
) -> Result<Vec<AccountsModel>, DbErr> {
    Accounts::find()
        .filter(AccountsBase::Column::ProposalId.eq(proposal_id))
        .all(db)
        .await
}
