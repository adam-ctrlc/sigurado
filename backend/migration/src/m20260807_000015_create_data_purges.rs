use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Clearing the data hides it rather than destroying it: one row here
        // marks the moment, and every history view ignores anything older. The
        // rows themselves stay exactly where they were, which is what makes the
        // action reversible and what keeps the promise that a reader event is
        // never edited or deleted.
        manager
            .create_table(
                Table::create()
                    .table(DataPurges::Table)
                    .if_not_exists()
                    .col(uuid(DataPurges::Id).primary_key())
                    .col(timestamp_with_time_zone(DataPurges::PurgedAt))
                    .col(uuid_null(DataPurges::PerformedBy))
                    .col(string_null(DataPurges::PerformedByName))
                    .col(string_null(DataPurges::Note))
                    // What was hidden, recorded at the time so the history reads
                    // the same later even as new rows arrive.
                    .col(integer(DataPurges::HiddenEvents).default(0))
                    .col(integer(DataPurges::HiddenCheckouts).default(0))
                    .col(integer(DataPurges::HiddenSessions).default(0))
                    .col(integer(DataPurges::HiddenLogins).default(0))
                    .col(integer(DataPurges::HiddenMessages).default(0))
                    .col(integer(DataPurges::HiddenEnrollments).default(0))
                    // Set when somebody puts it all back.
                    .col(timestamp_with_time_zone_null(DataPurges::UndoneAt))
                    .col(uuid_null(DataPurges::UndoneBy))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_data_purges_active")
                    .table(DataPurges::Table)
                    .col(DataPurges::UndoneAt)
                    .col(DataPurges::PurgedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DataPurges::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum DataPurges {
    Table,
    Id,
    PurgedAt,
    PerformedBy,
    PerformedByName,
    Note,
    HiddenEvents,
    HiddenCheckouts,
    HiddenSessions,
    HiddenLogins,
    HiddenMessages,
    HiddenEnrollments,
    UndoneAt,
    UndoneBy,
}
