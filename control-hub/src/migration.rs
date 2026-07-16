use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use serde::Deserialize;

const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/001_init_schema_migrations.sql"),
    include_str!("../migrations/002_init_logs.sql"),
    include_str!("../migrations/003_init_events.sql"),
    include_str!("../migrations/004_init_machine_schemas.sql"),
    include_str!("../migrations/005_init_machine_activity.sql"),
    include_str!("../migrations/006_init_machine_config_mutations.sql"),
    include_str!("../migrations/007_init_machine_state_mutations.sql"),
    include_str!("../migrations/008_init_machine_measurements.sql"),
];

pub async fn migrate(url: &str) -> Result<()> {
    let client = Client::default()
        .with_url(url)
        .with_user("default");

    println!("Checking migrations");

    let sql = "CREATE DATABASE IF NOT EXISTS control_hub";
    client.query(sql).execute().await?;

    let sql = "EXISTS TABLE control_hub.schema_migrations";
    let contains_table = client.query(sql).fetch_one::<u8>().await?;

    if contains_table != 1 {
        println!("Database not initalized, applying full migration");
        // not initialized at all. Do full migration
        return upgrade(client, 0).await
    }

    #[derive(Deserialize, Row)]
    pub struct MigrationEntry { 
        version: u64, 

        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        applied_at: DateTime<Utc>,
    }

    let entries = client
        .query("SELECT * FROM control_hub.schema_migrations ORDER BY version")
        .fetch_all::<MigrationEntry>()
        .await?;

    for (i, entry) in entries.iter().enumerate() {
        println!("Version {} was already applied at {}", entry.version, entry.applied_at);

        let expected = (i + 1) as u64;
        if entry.version != expected {
            bail!(
                "Invalid migration sequence: expected {}, got {}",
                expected,
                entry.version
            );
        }
    }

    let current: usize = entries.last().map_or(0, |v| v.version as usize);
    upgrade(client, current).await
}

async fn upgrade(client: Client, current: usize) -> anyhow::Result<()> {
    for (i, migration) in MIGRATIONS[current..].iter().enumerate() {
        let version = current + i + 1;
        println!("Version {version} was not applied. Applying ...");

        client
            .query(migration)
            .execute()
            .await?;

        client
            .query("INSERT INTO control_hub.schema_migrations (version) VALUES (?)")
            .bind(version)
            .execute()
            .await?;
    }

    Ok(())
}