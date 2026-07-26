use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::anyhow;
use clickhouse::Client;
use clickhouse::Row;
use qitech_framework_common::MachineIdentification;
use qitech_framework_common::MachineSchema;
use serde::Deserialize;

use crate::DatabaseConfig;

pub fn init_client(config: &DatabaseConfig) -> Client {
    let mut client = Client::default()
        .with_url(&config.url)
        .with_user(&config.user)
        // Enable JSON type and inserting JSON columns as string
        .with_setting("allow_experimental_json_type", "1")
        .with_setting("input_format_binary_read_json_as_string", "1");

    if let Some(password) = &config.password {
        client = client.with_password(password);
    }

    client
    // client.with_database(&config.name)
}

/// attempts to load the schema registry from the database
pub async fn init_schema_registry(
    client: &Client,
) -> anyhow::Result<Arc<BTreeMap<MachineIdentification, MachineSchema>>> {
    #[derive(Row, Deserialize)]
    struct SchemaRow {
        ident_vendor: u16,
        ident_machine: u16,
        content: String,
    }

    let fetched_schemas = client
        .query("SELECT * FROM machine_schemas")
        .fetch_all::<SchemaRow>()
        .await?;

    let mut registry = BTreeMap::new();

    for SchemaRow { content, .. } in fetched_schemas {
        let schema = MachineSchema::from_yaml_str(&content)?;

        if let Some(s) = registry.insert(schema.identification, schema) {
            return Err(anyhow!("Duplicate schema {}", s.identification));
        };
    }

    Ok(Arc::new(registry))
}

pub fn import_schemas(
    registry: Arc<BTreeMap<MachineIdentification, MachineSchema>>,
    incoming: Vec<String>,
) -> anyhow::Result<BTreeMap<MachineIdentification, MachineSchema>> {
    // lazily cloned copy-on-write map
    let mut new_schemas = (*registry).clone();

    for schema_data in incoming {
        let schema = MachineSchema::from_yaml_str(&schema_data)?;

        if let Some(s) = registry.get(&schema.identification) {
            if schema.revision != s.revision {
                return Err(anyhow!(
                    "schema revision mismatch for {}: expected {}",
                    schema.identification,
                    s.revision,
                ));
            }

            // duplicate, continue
            continue;
        }

        // not duplicate, put into registry
        new_schemas.insert(schema.identification, schema);
    }

    Ok(new_schemas)
}
