use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260719_000001_create_users::Users;
use crate::m20260730_000011_create_sms::{SmsMessages, SmsRecipients};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum UserContact {
    PhoneNumber,
}

#[derive(DeriveIden)]
enum RecipientUser {
    UserId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite applies DDL outside the migration transaction, so each step is
        // guarded: a half-applied run can be repeated safely.
        if !manager.has_column("users", "phone_number").await? {
            // The number now belongs to the person, not to a standalone alert
            // entry, so a custodian's number is stored once and reused.
            manager
                .alter_table(
                    Table::alter()
                        .table(Users::Table)
                        .add_column(string_null(UserContact::PhoneNumber))
                        .to_owned(),
                )
                .await?;
        }

        if manager.has_column("sms_recipients", "user_id").await? {
            return Ok(());
        }

        // Recipients become a subscription attached to a staff account. The old
        // label/number rows cannot be mapped to a user, so they are dropped:
        // detach the outbox first, since it points here.
        let db = manager.get_connection();
        db.execute_unprepared("UPDATE sms_messages SET recipient_id = NULL")
            .await?;
        manager
            .drop_table(Table::drop().table(SmsRecipients::Table).to_owned())
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SmsRecipients::Table)
                    .if_not_exists()
                    .col(uuid(SmsRecipients::Id).primary_key())
                    // One subscription per person.
                    .col(uuid_uniq(RecipientUser::UserId))
                    .col(boolean(SmsRecipients::NotifyCabinetOpened).default(true))
                    .col(boolean(SmsRecipients::NotifyAccessDenied).default(true))
                    .col(boolean(SmsRecipients::NotifyDoorOpened).default(false))
                    .col(boolean(SmsRecipients::IsActive).default(true))
                    .col(
                        timestamp_with_time_zone(SmsRecipients::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sms_recipients_user")
                            .from(SmsRecipients::Table, RecipientUser::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // No foreign key is re-added here: sms_messages already declares one against
        // sms_recipients from the original migration, SQLite resolves it by table
        // name, and it refuses to modify constraints on an existing table anyway.
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(UserContact::PhoneNumber)
                    .to_owned(),
            )
            .await
    }
}
