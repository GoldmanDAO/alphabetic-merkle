//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2
use serde::{Serialize, Deserialize};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub proposal_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub address: String,
    #[sea_orm(column_type = "Text")]
    pub balance: String,
    pub created_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::proposals::Entity",
        from = "Column::ProposalId",
        to = "super::proposals::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Proposals,
}

impl Related<super::proposals::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Proposals.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
