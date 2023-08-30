pub use sea_orm_migration::prelude::*;

mod m20230828_000000_setup_database;
mod m20230828_013249_create_address_domain;
mod m20230828_060306_create_proposal_table;
mod m20230828_071737_create_account_table;
mod m20230828_090601_link_account_proposal_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230828_000000_setup_database::Migration),
            Box::new(m20230828_013249_create_address_domain::Migration),
            Box::new(m20230828_060306_create_proposal_table::Migration),
            Box::new(m20230828_071737_create_account_table::Migration),
            Box::new(m20230828_090601_link_account_proposal_tables::Migration),
        ]
    }
}
