use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260719_000001_create_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LoginEvents::Table)
                    .if_not_exists()
                    .col(uuid(LoginEvents::Id).primary_key())
                    // Null when the identifier matched no account at all.
                    .col(uuid_null(LoginEvents::UserId))
                    .col(string(LoginEvents::Identifier))
                    .col(boolean(LoginEvents::Success))
                    .col(string_null(LoginEvents::Reason))
                    .col(string_null(LoginEvents::IpAddress))
                    .col(string_null(LoginEvents::UserAgent))
                    .col(
                        timestamp_with_time_zone(LoginEvents::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_login_events_user")
                            .from(LoginEvents::Table, LoginEvents::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_login_events_user")
                    .table(LoginEvents::Table)
                    .col(LoginEvents::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_login_events_created")
                    .table(LoginEvents::Table)
                    .col(LoginEvents::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LoginEvents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum LoginEvents {
    Table,
    Id,
    UserId,
    Identifier,
    Success,
    Reason,
    IpAddress,
    UserAgent,
    CreatedAt,
}
