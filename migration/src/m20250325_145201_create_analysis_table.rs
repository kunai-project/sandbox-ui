use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_samples_table::Sample;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(Analysis::Table)
                    .if_not_exists()
                    .col(uuid(Analysis::Uuid).primary_key().not_null())
                    .col(uuid(Analysis::SampleUuid).not_null())
                    .col(string(Analysis::SandboxName).not_null())
                    .col(ColumnDef::new(Analysis::SubmissionName).string().null())
                    .col(date_time(Analysis::Date).not_null())
                    .col(string(Analysis::SrcIp).not_null())
                    .col(string(Analysis::Status).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-analysis-sample_uuid")
                            .from(Analysis::Table, Analysis::SampleUuid)
                            .to(Sample::Table, Sample::Uuid),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(Analysis::submission_name_idx())
                    .table(Analysis::Table)
                    .col(Analysis::SubmissionName)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(Analysis::sample_uuid_idx())
                    .table(Analysis::Table)
                    .col(Analysis::SampleUuid)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_index(
                Index::drop()
                    .name(Analysis::submission_name_idx())
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().name(Analysis::sample_uuid_idx()).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Analysis::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Analysis {
    Table,
    Uuid,
    SandboxName,
    SampleUuid,
    SubmissionName,
    Date,
    SrcIp,
    Status,
}

impl Analysis {
    fn submission_name_idx() -> &'static str {
        "idx_submission_name"
    }

    fn sample_uuid_idx() -> &'static str {
        "idx_sample_uuid"
    }
}
