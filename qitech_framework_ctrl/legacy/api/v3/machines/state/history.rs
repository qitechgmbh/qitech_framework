use std::sync::Arc;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use chrono::{DateTime, Utc};
use axum::Json;
use axum::extract::{Path, Query, State};
use clickhouse::{Client, Row};
use control_core::{OperationResult, MachineIdentificationUnique};
use control_core::schema::{latest::{PropertyKind, Unit, state::Value}};
use crate::{SharedState, api::common::PropertyHistoryQuery};

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Entries {
    #[serde(rename = "Quantity")]
    String { data: Vec<Entry<String>> },
    Boolean { data: Vec<Entry<bool>> },
    Integer { data: Vec<Entry<i64>> },
    Float { data: Vec<Entry<f64>> },
    Percentage { data: Vec<Entry<f64>> },
    Fraction { data: Vec<Entry<f64>> },
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
    result: OperationResult,
}

pub(super) async fn get(
    State(state): State<Arc<SharedState>>,
    Path((slug, serial, property_name)): Path<(String, u16, String)>,
    Query(query): Query<PropertyHistoryQuery>,
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

    let mut current = &schema.state;
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
                query,
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
    kind: &PropertyKind<Value>,
    q: PropertyHistoryQuery,
) -> Result<Entries, String> {
    let value = match &kind {
        PropertyKind::Value(v) => v,
        PropertyKind::Group(_) => return Err("Not a value".into()),
    };

    match &value {
        Value::Enum(_) => Ok(Entries::String { 
            data: fetch_all::<String>(client, ident, name, "enum", q).await? 
        }),
        Value::String(_) => Ok(Entries::String { 
            data: fetch_all::<String>(client, ident, name, "string", q).await? 
        }),
        Value::Boolean(_) => Ok(Entries::Boolean { 
            data: fetch_all::<bool>(client, ident, name, "bool", q).await? 
        }),
        Value::Integer(_) => Ok(Entries::Integer { 
            data: fetch_all::<i64>(client, ident, name, "int", q).await? 
        }),
        Value::Float(_) => Ok(Entries::Float { 
            data: fetch_all::<f64>(client, ident, name, "float", q).await? 
        }),
        Value::Fraction(_) => Ok(Entries::Fraction { 
            data: fetch_all::<f64>(client, ident, name, "float", q).await? 
        }),
        Value::Percentage(_) => Ok(Entries::Percentage { 
            data: fetch_all::<f64>(client, ident, name, "float", q).await? 
        }),
        Value::Quantity { unit, .. } => Ok(Entries::Quantity { 
            unit: *unit,
            data: fetch_all::<f64>(client, ident, name, "float", q).await? 
        }),
    }
}

async fn fetch_all<T: DeserializeOwned + 'static>(
    client: &Client,
    ident: MachineIdentificationUnique,
    name: &str,
    column: &str,
    query: PropertyHistoryQuery,
) -> Result<Vec<Entry<T>>, String> {
    let sql = init_sql(column, &query)?;

    let mut q = client.query(&sql).bind(ident.to_u64()).bind(name);

    if let Some(from) = query.get_time_span()?.from {
        q = q.bind(dt_to_ch_datetime64_ms(from));
    }

    if let Some(to) = query.get_time_span()?.to {
        q = q.bind(dt_to_ch_datetime64_ms(to));
    }

    q = q.bind(query.get_limit());
    q.fetch_all::<Entry<T>>().await.map_err(|e| format!("{e}"))
}

fn init_sql(column: &str, query: &PropertyHistoryQuery) -> Result<String, String> {
    let mut sql = match &query.get_aggregation()? {
        Some(aggregation) => format!(
            r#"
            SELECT
                toDateTime64(toStartOfInterval(timestamp, {}), 3) AS timestamp,
                value_{column} AS value,
                origin,
                result
            FROM config_mutations
            WHERE identity = ?
            AND name = ?
            "#,
            aggregation.interval.to_ch(),
        ),
        None => format!(
            r#"
            SELECT timestamp, value_{column}, origin, result,
            FROM config_mutations
            WHERE identity = ?
            AND name = ?
        "#
        ),
    };

    // time filters
    if query.get_time_span()?.from.is_some() {
        sql.push_str(" AND timestamp >= toDateTime64(?, 3)");
    }

    if query.get_time_span()?.to.is_some() {
        sql.push_str(" AND timestamp <= toDateTime64(?, 3)");
    }

    // aggregation
    if query.get_aggregation()?.is_some() {
        sql.push_str("GROUP BY timestamp");
    }

    // ordering
    sql.push_str(" ORDER BY timestamp ");
    sql.push_str(query.ordering.unwrap_or_default().to_ch());

    // limit
    sql.push_str(" LIMIT ?");

    Ok(sql)
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
