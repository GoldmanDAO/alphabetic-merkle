use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "ALTER TABLE accounts 
                ADD CONSTRAINT fk_proposal_id
                FOREIGN KEY (proposal_id) 
                REFERENCES proposals (id);"
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE accounts DROP CONSTRAINT fk_proposal_id")
            .await?;

        Ok(())
    }
}
