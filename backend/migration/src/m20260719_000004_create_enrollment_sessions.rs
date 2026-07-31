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
                    .table(EnrollmentSessions::Table)
                    .if_not_exists()
                    .col(uuid(EnrollmentSessions::Id).primary_key())
                    .col(uuid_null(EnrollmentSessions::UserId))
                    .col(uuid(EnrollmentSessions::DeviceId))
                    .col(string(EnrollmentSessions::Code))
                    .col(string(EnrollmentSessions::Status))
                    .col(string_null(EnrollmentSessions::PendingFingerToken))
                    .col(
                        timestamp_with_time_zone(EnrollmentSessions::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone(EnrollmentSessions::CodeExpiresAt))
                    .col(timestamp_with_time_zone_null(EnrollmentSessions::BoundAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_enrollment_user")
                            .from(EnrollmentSessions::Table, EnrollmentSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_enrollment_device")
                            .from(EnrollmentSessions::Table, EnrollmentSessions::DeviceId)
                            .to(Devices::Table, Devices::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_enrollment_code")
                    .table(EnrollmentSessions::Table)
                    .col(EnrollmentSessions::Code)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_enrollment_device")
                    .table(EnrollmentSessions::Table)
                    .col(EnrollmentSessions::DeviceId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EnrollmentSessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum EnrollmentSessions {
    Table,
    Id,
    UserId,
    DeviceId,
    Code,
    Status,
    PendingFingerToken,
    CreatedAt,
    CodeExpiresAt,
    BoundAt,
}
