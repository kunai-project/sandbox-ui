use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(Sample::Table)
                    .if_not_exists()
                    .col(uuid(Sample::Uuid).primary_key().not_null())
                    .col(string(Sample::Md5).not_null())
                    .index(Index::create().name("idx_md5").col(Sample::Md5).unique())
                    .col(string(Sample::Sha1).not_null())
                    .index(Index::create().name("idx_sha1").col(Sample::Sha1).unique())
                    .col(string(Sample::Sha256).not_null())
                    .index(
                        Index::create()
                            .name("idx_sha256")
                            .col(Sample::Sha256)
                            .unique(),
                    )
                    .col(string(Sample::Sha512).not_null())
                    .index(
                        Index::create()
                            .name("idx_sha512")
                            .col(Sample::Sha512)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Sample::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Sample {
    Table,
    Uuid,
    Md5,
    Sha1,
    Sha256,
    Sha512,
}
