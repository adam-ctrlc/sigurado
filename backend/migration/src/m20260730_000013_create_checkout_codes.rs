use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260719_000001_create_users::Users;
use crate::m20260719_000002_create_devices::Devices;
use crate::m20260719_000005_create_access_sessions::AccessSessions;
use crate::m20260719_000007_create_checkouts::Checkouts;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // The cabinet shows a code when it opens, and that code is what lets the
        // person record what they took. It ties the written record to a specific
        // physical opening rather than to whoever happens to be signed in.
        manager
            .create_table(
                Table::create()
                    .table(CheckoutCodes::Table)
                    .if_not_exists()
                    .col(uuid(CheckoutCodes::Id).primary_key())
                    .col(string_uniq(CheckoutCodes::Code))
                    .col(uuid(CheckoutCodes::UserId))
                    .col(uuid_null(CheckoutCodes::DeviceId))
                    .col(uuid_null(CheckoutCodes::AccessSessionId))
                    .col(timestamp_with_time_zone(CheckoutCodes::ExpiresAt))
                    .col(timestamp_with_time_zone_null(CheckoutCodes::UsedAt))
                    .col(uuid_null(CheckoutCodes::CheckoutId))
                    .col(
                        timestamp_with_time_zone(CheckoutCodes::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_checkout_codes_user")
                            .from(CheckoutCodes::Table, CheckoutCodes::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_checkout_codes_device")
                            .from(CheckoutCodes::Table, CheckoutCodes::DeviceId)
                            .to(Devices::Table, Devices::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_checkout_codes_session")
                            .from(CheckoutCodes::Table, CheckoutCodes::AccessSessionId)
                            .to(AccessSessions::Table, AccessSessions::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_checkout_codes_checkout")
                            .from(CheckoutCodes::Table, CheckoutCodes::CheckoutId)
                            .to(Checkouts::Table, Checkouts::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_checkout_codes_user")
                    .table(CheckoutCodes::Table)
                    .col(CheckoutCodes::UserId)
                    .col(CheckoutCodes::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CheckoutCodes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum CheckoutCodes {
    Table,
    Id,
    Code,
    UserId,
    DeviceId,
    AccessSessionId,
    ExpiresAt,
    UsedAt,
    CheckoutId,
    CreatedAt,
}
