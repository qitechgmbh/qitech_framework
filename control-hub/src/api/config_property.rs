use crate::{SharedState, api::types::HistoryArgs};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use control_core::{
    ConfigMutationResult, MachineIdentificationUnique,
    schema::{
        latest::{PropertyKind, config::ValueV2},
        v1_0::Unit,
    },
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::sync::Arc;

#[derive(Serialize)]
#[serde(untagged)]
pub(super) enum Entries {
    String { data: Vec<Entry<String>> },
    Boolean { data: Vec<Entry<bool>> },
    Integer { data: Vec<Entry<i64>> },
    Float { data: Vec<Entry<f64>> },
    Quantity { 
        unit: Unit, 
        data: Vec<Entry<f64>>,
    },
}

#[derive(Serialize, Deserialize, Row)]
pub(super) struct Entry<T> {
    timestamp: DateTime<Utc>,
    value: T,
    origin: u64,
    result: ConfigMutationResult,
}

pub(super) async fn get(
    State(state): State<Arc<SharedState>>,
    Path((slug, serial, property_name)): Path<(String, u16, String)>,
    Query(args): Query<HistoryArgs>,
) -> Result<Json<Entries>, String> {
    // ensure we have such a machine type defined in the schemas
    let schemas = state.schemas.load();
    let Some((ident, schema)) = schemas.iter().find(|(_, s)| s.name == slug) else {
        return Err("No such machine slug".into());
    };

    let ident_unique = MachineIdentificationUnique {
        vendor: ident.vendor,
        machine: ident.machine,
        serial,
    };

    let path: Vec<&str> = property_name.split('.').collect();

    let mut current = &schema.config;
    for (index, part) in path.iter().enumerate() {
        let Some(prop) = current.get(*part) else {
            println!("not found at {}", part);
            break;
        };

        if index == path.len() - 1 {
            let entries = read_entries(
                &state.client,
                ident_unique, 
                &property_name,
                &prop.kind, 
                args,
            ).await?;

            return Ok(axum::Json(entries));
        }

        let PropertyKind::Group(children) = &prop.kind else {
            return Err("No such path".into());
        };

        current = children;
    }

    Err("No such property".into())
}

async fn read_entries(
    client: &Client,
    ident: MachineIdentificationUnique,
    name: &str,
    kind: &PropertyKind<ValueV2>,
    args: HistoryArgs,
) -> Result<Entries, String> {
    let value = match &kind {
        PropertyKind::Value(v) => v,
        PropertyKind::Group(_) => return Err("Not a value".into()),
    };

    match &value {
        ValueV2::Enum(_) => Ok(Entries::String { 
            data: fetch_all::<String>(client, ident, name, "enum", args).await? 
        }),
        ValueV2::String(_) => Ok(Entries::String { 
            data: fetch_all::<String>(client, ident, name, "string", args).await? 
        }),
        ValueV2::Boolean(_) => Ok(Entries::Boolean { 
            data: fetch_all::<bool>(client, ident, name, "bool", args).await? 
        }),
        ValueV2::Integer(_) => Ok(Entries::Integer { 
            data: fetch_all::<i64>(client, ident, name, "int", args).await? 
        }),
        ValueV2::Float(_) => Ok(Entries::Float { 
            data: fetch_all::<f64>(client, ident, name, "float", args).await? 
        }),
        ValueV2::Quantity { unit, .. } => Ok(Entries::Quantity { 
            unit: *unit,
            data: fetch_all::<f64>(client, ident, name, "float", args).await? 
        }),
    }
}

async fn fetch_all<T: DeserializeOwned>(
    client: &Client,
    ident: MachineIdentificationUnique,
    name: &str,
    column: &str,
    args: HistoryArgs,
) -> Result<Vec<Entry<T>>, String> {
    let sql = init_sql(column, &args);

    let mut query = client.query(&sql).bind(ident.to_u64()).bind(name);

    if let Some(from) = args.time_span.from {
        query = query.bind(dt_to_ch_datetime64_ms(from));
    }

    if let Some(to) = args.time_span.to {
        query = query.bind(dt_to_ch_datetime64_ms(to));
    }

    query = query.bind(args.limit);
    query.fetch_all::<Entry<T>>().await.map_err(|e| format!("{e}"))
}

fn init_sql(column: &str, args: &HistoryArgs) -> String {
    let mut sql = match &args.aggregation {
        Some(aggregation) => format!(
            r#"
            SELECT
                toDateTime64(toStartOfInterval(ts, {}), 3) AS ts,
                value_{column} AS value
            FROM config_mutations
            WHERE identity = ?
            AND name = ?
            "#,
            aggregation.interval.to_ch(),
        ),
        None => format!(
            r#"
            SELECT ts, value_{column},
            FROM config_mutations
            WHERE identity = ?
            AND name = ?
        "#
        ),
    };

    // time filters
    if args.time_span.from.is_some() {
        sql.push_str(" AND ts >= toDateTime64(?, 3)");
    }

    if args.time_span.to.is_some() {
        sql.push_str(" AND ts <= toDateTime64(?, 3)");
    }

    // aggregation
    if args.aggregation.is_some() {
        sql.push_str("GROUP BY ts");
    }

    // ordering
    sql.push_str(" ORDER BY ts ");
    sql.push_str(args.ordering.to_ch());

    // limit
    sql.push_str(" LIMIT ?");

    sql
}

fn dt_to_ch_datetime64_ms(dt: DateTime<Utc>) -> String {
    let secs = dt.timestamp();
    let nanos = dt.timestamp_subsec_nanos();

    // convert nanos -> fractional seconds (up to 9 digits)
    let frac = nanos as f64 / 1_000_000_000.0;
    let value = secs as f64 + frac;

    // format with 3 decimals for DateTime64(3)
    format!("{:.3}", value)
}
