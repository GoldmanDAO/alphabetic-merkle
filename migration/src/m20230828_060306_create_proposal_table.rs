use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "CREATE TABLE proposals (
                id SERIAL PRIMARY KEY,
                author address NOT NULL,
                block_number BIGINT NOT NULL,
                ipfs_hash TEXT NOT NULL,
                root_hash TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT NOW()
            );"
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE proposals")
            .await?;

        Ok(())
    }
}
