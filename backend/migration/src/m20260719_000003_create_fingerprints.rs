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
                    .table(Fingerprints::Table)
                    .if_not_exists()
                    .col(uuid(Fingerprints::Id).primary_key())
                    .col(uuid(Fingerprints::UserId))
                    .col(string_uniq(Fingerprints::FingerToken))
                    .col(string_null(Fingerprints::Label))
                    .col(uuid_null(Fingerprints::EnrolledViaDeviceId))
                    .col(
                        timestamp_with_time_zone(Fingerprints::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fingerprints_user")
                            .from(Fingerprints::Table, Fingerprints::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_fingerprints_device")
                            .from(Fingerprints::Table, Fingerprints::EnrolledViaDeviceId)
                            .to(Devices::Table, Devices::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_fingerprints_user")
                    .table(Fingerprints::Table)
                    .col(Fingerprints::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Fingerprints::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Fingerprints {
    Table,
    Id,
    UserId,
    FingerToken,
    Label,
    EnrolledViaDeviceId,
    CreatedAt,
}
