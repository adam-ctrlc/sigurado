use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260719_000001_create_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum UserNames {
    FirstName,
    MiddleName,
    LastName,
    Suffix,
    LastLoginAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite only takes one ADD COLUMN per statement, so these go one at a time.
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(string(UserNames::FirstName).default(""))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(string_null(UserNames::MiddleName))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(string(UserNames::LastName).default(""))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(string_null(UserNames::Suffix))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(timestamp_with_time_zone_null(UserNames::LastLoginAt))
                    .to_owned(),
            )
            .await?;

        // Split the old single-field name: everything before the first space is
        // the given name, the rest is the family name. Anyone whose real name
        // needs a middle name or suffix can be corrected in the admin UI.
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            UPDATE users SET
                first_name = CASE
                    WHEN instr(display_name, ' ') > 0
                        THEN substr(display_name, 1, instr(display_name, ' ') - 1)
                    ELSE display_name
                END,
                last_name = CASE
                    WHEN instr(display_name, ' ') > 0
                        THEN trim(substr(display_name, instr(display_name, ' ') + 1))
                    ELSE ''
                END
            "#,
        )
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::DisplayName)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(string(Users::DisplayName).default(""))
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            "UPDATE users SET display_name = trim(first_name || ' ' || last_name)",
        )
        .await?;

        for column in [
            UserNames::FirstName,
            UserNames::MiddleName,
            UserNames::LastName,
            UserNames::Suffix,
            UserNames::LastLoginAt,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(Users::Table)
                        .drop_column(column)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}
