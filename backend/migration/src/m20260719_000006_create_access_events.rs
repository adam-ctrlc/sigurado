use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260719_000001_create_users::Users;
use crate::m20260719_000002_create_devices::Devices;
use crate::m20260719_000004_create_enrollment_sessions::EnrollmentSessions;
use crate::m20260719_000005_create_access_sessions::AccessSessions;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AccessEvents::Table)
                    .if_not_exists()
                    .col(uuid(AccessEvents::Id).primary_key())
                    .col(string(AccessEvents::EventType))
                    .col(string(AccessEvents::Decision))
                    .col(uuid_null(AccessEvents::DeviceId))
                    .col(uuid_null(AccessEvents::UserId))
                    .col(string_null(AccessEvents::FingerToken))
                    .col(uuid_null(AccessEvents::AccessSessionId))
                    .col(uuid_null(AccessEvents::EnrollmentSessionId))
                    .col(string_null(AccessEvents::Message))
                    .col(
                        timestamp_with_time_zone(AccessEvents::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_device")
                            .from(AccessEvents::Table, AccessEvents::DeviceId)
                            .to(Devices::Table, Devices::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_user")
                            .from(AccessEvents::Table, AccessEvents::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_access_session")
                            .from(AccessEvents::Table, AccessEvents::AccessSessionId)
                            .to(AccessSessions::Table, AccessSessions::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_enrollment_session")
                            .from(AccessEvents::Table, AccessEvents::EnrollmentSessionId)
                            .to(EnrollmentSessions::Table, EnrollmentSessions::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_event_created_at")
                    .table(AccessEvents::Table)
                    .col(AccessEvents::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_event_type")
                    .table(AccessEvents::Table)
                    .col(AccessEvents::EventType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_event_decision")
                    .table(AccessEvents::Table)
                    .col(AccessEvents::Decision)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AccessEvents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum AccessEvents {
    Table,
    Id,
    EventType,
    Decision,
    DeviceId,
    UserId,
    FingerToken,
    AccessSessionId,
    EnrollmentSessionId,
    Message,
    CreatedAt,
}
