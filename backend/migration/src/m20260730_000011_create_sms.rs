use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260719_000002_create_devices::Devices;
use crate::m20260719_000006_create_access_events::AccessEvents;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Which node carries the SIM800L. Only that one may drain the queue, so a
        // second node polling cannot send every alert twice.
        manager
            .alter_table(
                Table::alter()
                    .table(Devices::Table)
                    .add_column(boolean(SmsColumns::SmsCapable).default(false))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SmsRecipients::Table)
                    .if_not_exists()
                    .col(uuid(SmsRecipients::Id).primary_key())
                    .col(string(SmsRecipients::Label))
                    .col(string(SmsRecipients::PhoneNumber))
                    .col(boolean(SmsRecipients::NotifyCabinetOpened).default(true))
                    .col(boolean(SmsRecipients::NotifyAccessDenied).default(true))
                    .col(boolean(SmsRecipients::NotifyDoorOpened).default(false))
                    .col(boolean(SmsRecipients::IsActive).default(true))
                    .col(
                        timestamp_with_time_zone(SmsRecipients::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SmsMessages::Table)
                    .if_not_exists()
                    .col(uuid(SmsMessages::Id).primary_key())
                    // Kept even if the recipient is later removed, so the outbox
                    // stays readable.
                    .col(uuid_null(SmsMessages::RecipientId))
                    .col(string(SmsMessages::PhoneNumber))
                    .col(string(SmsMessages::Body))
                    .col(string(SmsMessages::Status).default("pending"))
                    .col(integer(SmsMessages::Attempts).default(0))
                    .col(string_null(SmsMessages::Error))
                    .col(uuid_null(SmsMessages::AccessEventId))
                    .col(uuid_null(SmsMessages::SentByDeviceId))
                    .col(
                        timestamp_with_time_zone(SmsMessages::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone_null(SmsMessages::SentAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sms_messages_recipient")
                            .from(SmsMessages::Table, SmsMessages::RecipientId)
                            .to(SmsRecipients::Table, SmsRecipients::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sms_messages_event")
                            .from(SmsMessages::Table, SmsMessages::AccessEventId)
                            .to(AccessEvents::Table, AccessEvents::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sms_messages_device")
                            .from(SmsMessages::Table, SmsMessages::SentByDeviceId)
                            .to(Devices::Table, Devices::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sms_messages_status")
                    .table(SmsMessages::Table)
                    .col(SmsMessages::Status)
                    .col(SmsMessages::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SmsMessages::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SmsRecipients::Table).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Devices::Table)
                    .drop_column(SmsColumns::SmsCapable)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum SmsColumns {
    SmsCapable,
}

#[derive(DeriveIden)]
pub enum SmsRecipients {
    Table,
    Id,
    Label,
    PhoneNumber,
    NotifyCabinetOpened,
    NotifyAccessDenied,
    NotifyDoorOpened,
    IsActive,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum SmsMessages {
    Table,
    Id,
    RecipientId,
    PhoneNumber,
    Body,
    Status,
    Attempts,
    Error,
    AccessEventId,
    SentByDeviceId,
    CreatedAt,
    SentAt,
}
