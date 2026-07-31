use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260719_000001_create_users::Users;
use crate::m20260719_000002_create_devices::Devices;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AccessSessions::Table)
                    .if_not_exists()
                    .col(uuid(AccessSessions::Id).primary_key())
                    .col(uuid(AccessSessions::UserId))
                    .col(uuid(AccessSessions::DoorDeviceId))
                    .col(
                        timestamp_with_time_zone(AccessSessions::OpenedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone(AccessSessions::ExpiresAt))
                    .col(string(AccessSessions::Status).default("active"))
                    .col(uuid_null(AccessSessions::ConsumedByBoxDeviceId))
                    .col(timestamp_with_time_zone_null(AccessSessions::ClosedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_access_session_user")
                            .from(AccessSessions::Table, AccessSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_access_session_door")
                            .from(AccessSessions::Table, AccessSessions::DoorDeviceId)
                            .to(Devices::Table, Devices::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_access_session_user")
                    .table(AccessSessions::Table)
                    .col(AccessSessions::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_access_session_status_expires")
                    .table(AccessSessions::Table)
                    .col(AccessSessions::Status)
                    .col(AccessSessions::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AccessSessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum AccessSessions {
    Table,
    Id,
    UserId,
    DoorDeviceId,
    OpenedAt,
    ExpiresAt,
    Status,
    ConsumedByBoxDeviceId,
    ClosedAt,
}
