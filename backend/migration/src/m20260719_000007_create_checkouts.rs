use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260719_000001_create_users::Users;
use crate::m20260719_000002_create_devices::Devices;
use crate::m20260719_000005_create_access_sessions::AccessSessions;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Checkouts::Table)
                    .if_not_exists()
                    .col(uuid(Checkouts::Id).primary_key())
                    .col(uuid(Checkouts::UserId))
                    .col(uuid_null(Checkouts::AccessSessionId))
                    .col(uuid_null(Checkouts::BoxDeviceId))
                    .col(text(Checkouts::Note))
                    .col(string_null(Checkouts::PhotoPath))
                    .col(string_null(Checkouts::PhotoMime))
                    .col(
                        timestamp_with_time_zone(Checkouts::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_checkout_user")
                            .from(Checkouts::Table, Checkouts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_checkout_access_session")
                            .from(Checkouts::Table, Checkouts::AccessSessionId)
                            .to(AccessSessions::Table, AccessSessions::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_checkout_box_device")
                            .from(Checkouts::Table, Checkouts::BoxDeviceId)
                            .to(Devices::Table, Devices::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_checkout_user")
                    .table(Checkouts::Table)
                    .col(Checkouts::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_checkout_created_at")
                    .table(Checkouts::Table)
                    .col(Checkouts::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Checkouts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Checkouts {
    Table,
    Id,
    UserId,
    AccessSessionId,
    BoxDeviceId,
    Note,
    PhotoPath,
    PhotoMime,
    CreatedAt,
}
