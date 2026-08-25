use anyhow::bail;
use chrono::DateTime;
use chrono::Utc;
use clickhouse::Client;
use clickhouse::Row;
use serde::Deserialize;
use tracing::info;
use tracing::warn;

const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/001_init_schema_migrations.sql"),
    include_str!("../migrations/002_init_logs.sql"),
    include_str!("../migrations/003_init_events.sql"),
    include_str!("../migrations/004_init_machine_schemas.sql"),
    include_str!("../migrations/005_init_machine_activity.sql"),
    include_str!("../migrations/006_init_machine_config_mutations.sql"),
    include_str!("../migrations/007_init_machine_state_mutations.sql"),
    include_str!("../migrations/008_init_machine_measurements.sql"),
    include_str!("../migrations/009_init_machine_command_calls.sql"),
    include_str!("../migrations/010_init_runtime_transactions.sql"),
];

#[derive(Deserialize, Row)]
pub struct MigrationEntry {
    version: u64,

    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    applied_at: DateTime<Utc>,
}

#[tracing::instrument(
    name = "migration::validate",
    skip(client),
    fields(database = "control_hub"),
    err
)]
pub async fn validate(client: &Client) -> anyhow::Result<()> {
    // --- ensure database exists ---
    let contains_db = client
        .query("EXISTS DATABASE control_hub")
        .fetch_one::<u8>()
        .await?;

    if contains_db != 1 {
        warn!("Database is missing");
        bail!("Database 'control_hub' not present");
    }

    let contains_table = client
        .query("EXISTS TABLE control_hub.schema_migrations")
        .fetch_one::<u8>()
        .await?;

    if contains_table != 1 {
        info!("Migration table is missing; applying initial migration");
        return upgrade(client, 0).await;
    }

    let last_version = client
        .query("SELECT * FROM control_hub.schema_migrations ORDER BY version DESC LIMIT 1")
        .fetch_one::<MigrationEntry>()
        .await?;

    let expected = MIGRATIONS.len() as u64;

    if last_version.version != expected {
        warn!(
            current_version = last_version.version,
            expected_version = expected,
            "Database migration version mismatch"
        );

        bail!(
            "database 'control_hub' out of date. On version {}, but expected {}",
            last_version.version,
            expected
        );
    }

    info!(
        version = last_version.version,
        "Database migration validation completed"
    );

    Ok(())
}

#[tracing::instrument(
    name = "migration::execute",
    skip(client),
    fields(database = "control_hub")
)]
pub async fn execute(client: &Client) -> anyhow::Result<()> {
    info!("Checking database migrations");

    client
        .query("CREATE DATABASE IF NOT EXISTS control_hub")
        .execute()
        .await?;

    let initialized = client
        .query("EXISTS TABLE control_hub.schema_migrations")
        .fetch_one::<u8>()
        .await?
        == 1;

    if !initialized {
        info!("Database is not initialized; applying full migration");
        return upgrade(client, 0).await;
    }

    let entries = client
        .query("SELECT * FROM control_hub.schema_migrations ORDER BY version")
        .fetch_all::<MigrationEntry>()
        .await?;

    for (i, entry) in entries.iter().enumerate() {
        info!(
            version = entry.version,
            applied_at = %entry.applied_at,
            "Migration already applied"
        );

        let expected = (i + 1) as u64;
        if entry.version != expected {
            bail!(
                "Invalid migration sequence: expected {}, got {}",
                expected,
                entry.version
            );
        }
    }

    let current = entries.last().map_or(0, |v| v.version as usize);
    info!(current_version = current, "Checking for pending migrations");
    upgrade(client, current).await
}

async fn upgrade(client: &Client, current: usize) -> anyhow::Result<()> {
    let pending = MIGRATIONS.len().saturating_sub(current);

    info!(pending, "Applying migrations");

    for (i, migration) in MIGRATIONS[current..].iter().enumerate() {
        let version = current + i + 1;

        info!(version, "Applying migration");

        client.query(migration).execute().await?;

        client
            .query("INSERT INTO control_hub.schema_migrations (version) VALUES (?)")
            .bind(version)
            .execute()
            .await?;

        info!(version, "Migration applied");
    }

    info!("Schema is up to date");

    Ok(())
}
