use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260719_000002_create_devices::Devices;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // One row per continuous stretch of contact with a node, rather than one
        // row per heartbeat: a node beating every 30 seconds would otherwise
        // write ~2,900 rows a day for nothing.
        manager
            .create_table(
                Table::create()
                    .table(DeviceConnections::Table)
                    .if_not_exists()
                    .col(uuid(DeviceConnections::Id).primary_key())
                    .col(uuid(DeviceConnections::DeviceId))
                    .col(timestamp_with_time_zone(DeviceConnections::ConnectedAt))
                    .col(timestamp_with_time_zone(DeviceConnections::LastSeenAt))
                    .col(integer(DeviceConnections::Heartbeats).default(1))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_device_connections_device")
                            .from(DeviceConnections::Table, DeviceConnections::DeviceId)
                            .to(Devices::Table, Devices::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_device_connections_device")
                    .table(DeviceConnections::Table)
                    .col(DeviceConnections::DeviceId)
                    .col(DeviceConnections::ConnectedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DeviceConnections::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum DeviceConnections {
    Table,
    Id,
    DeviceId,
    ConnectedAt,
    LastSeenAt,
    Heartbeats,
}
