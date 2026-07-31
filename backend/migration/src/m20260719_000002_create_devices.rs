use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Devices::Table)
                    .if_not_exists()
                    .col(uuid(Devices::Id).primary_key())
                    .col(string_uniq(Devices::Name))
                    .col(string(Devices::Kind))
                    .col(string(Devices::SecretHash))
                    .col(timestamp_with_time_zone_null(Devices::LastSeenAt))
                    .col(boolean(Devices::IsActive).default(true))
                    .col(
                        timestamp_with_time_zone(Devices::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Devices::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Devices {
    Table,
    Id,
    Name,
    Kind,
    SecretHash,
    LastSeenAt,
    IsActive,
    CreatedAt,
}
