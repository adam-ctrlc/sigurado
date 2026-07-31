pub use sea_orm_migration::prelude::*;

mod m20260719_000001_create_users;
mod m20260719_000002_create_devices;
mod m20260719_000003_create_fingerprints;
mod m20260719_000004_create_enrollment_sessions;
mod m20260719_000005_create_access_sessions;
mod m20260719_000006_create_access_events;
mod m20260719_000007_create_checkouts;
mod m20260729_000008_split_user_names;
mod m20260729_000009_create_login_events;
mod m20260730_000010_create_device_connections;
mod m20260730_000011_create_sms;
mod m20260730_000012_sms_recipients_are_users;
mod m20260730_000013_create_checkout_codes;
mod m20260730_000014_create_sheet_syncs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260719_000001_create_users::Migration),
            Box::new(m20260719_000002_create_devices::Migration),
            Box::new(m20260719_000003_create_fingerprints::Migration),
            Box::new(m20260719_000004_create_enrollment_sessions::Migration),
            Box::new(m20260719_000005_create_access_sessions::Migration),
            Box::new(m20260719_000006_create_access_events::Migration),
            Box::new(m20260719_000007_create_checkouts::Migration),
            Box::new(m20260729_000008_split_user_names::Migration),
            Box::new(m20260729_000009_create_login_events::Migration),
            Box::new(m20260730_000010_create_device_connections::Migration),
            Box::new(m20260730_000011_create_sms::Migration),
            Box::new(m20260730_000012_sms_recipients_are_users::Migration),
            Box::new(m20260730_000013_create_checkout_codes::Migration),
            Box::new(m20260730_000014_create_sheet_syncs::Migration),
        ]
    }
}
