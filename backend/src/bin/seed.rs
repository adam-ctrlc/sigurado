use anyhow::Context;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use backend::{
    auth::password::hash_password,
    config::Config,
    entities::{
        devices, prelude::*, sea_orm_active_enums::DeviceKind, sea_orm_active_enums::Role, users,
    },
    util,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cfg = Config::from_env()?;
    let db = Database::connect(&cfg.database_url)
        .await
        .context("connecting to the database")?;

    seed_admin(&db).await?;
    println!();
    seed_student(&db).await?;
    println!();
    seed_faculty(&db).await?;
    println!();
    seed_device(&db, "front-door", DeviceKind::Door).await?;
    seed_device(&db, "cabinet-1", DeviceKind::Box).await?;

    println!("\nSeeding complete. Paste each device secret into its simulator panel.");
    Ok(())
}

async fn seed_admin(db: &DatabaseConnection) -> anyhow::Result<()> {
    let username = env("SEED_ADMIN_USERNAME", "admin");
    let email = env("SEED_ADMIN_EMAIL", "admin@sigurado.local");
    let password = env("SEED_ADMIN_PASSWORD", "admin12345");
    let first_name = env("SEED_ADMIN_FIRST_NAME", "Lab");
    let last_name = env("SEED_ADMIN_LAST_NAME", "Administrator");

    let existing = Users::find()
        .filter(users::Column::Username.eq(&username))
        .one(db)
        .await?;
    if existing.is_some() {
        println!("Admin '{username}' already exists; leaving it untouched.");
        return Ok(());
    }

    let now = util::now();
    users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email.clone()),
        username: Set(username.clone()),
        password_hash: Set(hash_password(&password)?),
        first_name: Set(first_name),
        middle_name: Set(None),
        last_name: Set(last_name),
        suffix: Set(None),
        phone_number: Set(Some(env("SEED_ADMIN_PHONE", "+639171234567"))),
        role: Set(Role::Admin),
        is_active: Set(true),
        last_login_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;

    println!("Created admin account:");
    println!("  username: {username}");
    println!("  email:    {email}");
    println!("  password: {password}");
    Ok(())
}

/// An ordinary student account, so the non-admin views can be tried out.
async fn seed_student(db: &DatabaseConnection) -> anyhow::Result<()> {
    let username = env("SEED_STUDENT_USERNAME", "student");
    let email = env("SEED_STUDENT_EMAIL", "student@sigurado.local");
    let password = env("SEED_STUDENT_PASSWORD", "student12345");

    let existing = Users::find()
        .filter(users::Column::Username.eq(&username))
        .one(db)
        .await?;
    if existing.is_some() {
        println!("Student '{username}' already exists; leaving it untouched.");
        return Ok(());
    }

    let now = util::now();
    users::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email.clone()),
        username: Set(username.clone()),
        password_hash: Set(hash_password(&password)?),
        first_name: Set(env("SEED_STUDENT_FIRST_NAME", "Juan")),
        middle_name: Set(Some(env("SEED_STUDENT_MIDDLE_NAME", "Ramos"))),
        last_name: Set(env("SEED_STUDENT_LAST_NAME", "Dela Cruz")),
        suffix: Set(None),
        // Students never receive alerts, so no number is seeded.
        phone_number: Set(None),
        role: Set(Role::Student),
        is_active: Set(true),
        last_login_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;

    println!("Created student account:");
    println!("  username: {username}");
    println!("  email:    {email}");
    println!("  password: {password}");
    Ok(())
}

/// A few faculty accounts, so the SMS recipient picker and the role filters have
/// something real to work with. Numbers are seeded because only staff receive
/// alerts, and a subscription without a number cannot send anything.
async fn seed_faculty(db: &DatabaseConnection) -> anyhow::Result<()> {
    let password = env("SEED_FACULTY_PASSWORD", "Faculty@123");
    let people = [
        ("maria.s.reyes", "Maria", "Santos", "Reyes", "+639171112233"),
        (
            "carlo.d.mendoza",
            "Carlo",
            "Diaz",
            "Mendoza",
            "+639172223344",
        ),
        (
            "elena.c.bautista",
            "Elena",
            "Cruz",
            "Bautista",
            "+639173334455",
        ),
    ];

    let mut created = 0;
    for (username, first, middle, last, phone) in people {
        let existing = Users::find()
            .filter(users::Column::Username.eq(username))
            .one(db)
            .await?;
        if existing.is_some() {
            continue;
        }

        let now = util::now();
        users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(format!("{username}@sigurado.local")),
            username: Set(username.to_owned()),
            password_hash: Set(hash_password(&password)?),
            first_name: Set(first.to_owned()),
            middle_name: Set(Some(middle.to_owned())),
            last_name: Set(last.to_owned()),
            suffix: Set(None),
            phone_number: Set(Some(phone.to_owned())),
            role: Set(Role::Faculty),
            is_active: Set(true),
            last_login_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await?;
        created += 1;
    }

    if created == 0 {
        println!("Faculty accounts already exist; leaving them untouched.");
    } else {
        println!("Created {created} faculty account(s), password: {password}");
        for (username, ..) in people {
            println!("  {username}");
        }
    }
    Ok(())
}

async fn seed_device(db: &DatabaseConnection, name: &str, kind: DeviceKind) -> anyhow::Result<()> {
    let existing = Devices::find()
        .filter(devices::Column::Name.eq(name))
        .one(db)
        .await?;
    if existing.is_some() {
        println!(
            "Device '{name}' already exists; secret is not recoverable. Rotate it from the admin UI to get a new one."
        );
        return Ok(());
    }

    let secret = util::random_token(40);
    let device = devices::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name.to_owned()),
        kind: Set(kind),
        secret_hash: Set(hash_password(&secret)?),
        last_seen_at: Set(None),
        is_active: Set(true),
        // The SIM800L lives with the cabinet node in this build.
        sms_capable: Set(kind == DeviceKind::Box),
        created_at: Set(util::now()),
    }
    .insert(db)
    .await?;

    println!("Created {kind:?} device '{name}':");
    println!("  device_id:     {}", device.id);
    println!("  device_secret: {secret}");
    Ok(())
}

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}
