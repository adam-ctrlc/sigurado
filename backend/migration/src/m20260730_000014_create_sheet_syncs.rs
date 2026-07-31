use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // One row per tab in the spreadsheet, recording how far the export got.
        // Without it a restart would append every row again and the sheet would
        // fill with duplicates.
        manager
            .create_table(
                Table::create()
                    .table(SheetSyncs::Table)
                    .if_not_exists()
                    .col(string(SheetSyncs::Tab).primary_key())
                    // Opaque "timestamp|id" of the last row sent, so ordering is
                    // stable even when two rows share a timestamp.
                    .col(string_null(SheetSyncs::LastKey))
                    .col(integer(SheetSyncs::RowsSent).default(0))
                    .col(timestamp_with_time_zone_null(SheetSyncs::LastSyncedAt))
                    .col(string_null(SheetSyncs::LastError))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SheetSyncs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum SheetSyncs {
    Table,
    Tab,
    LastKey,
    RowsSent,
    LastSyncedAt,
    LastError,
}
