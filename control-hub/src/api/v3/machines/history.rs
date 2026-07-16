/// endpoint for retrieving history of a machine

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use axum::Json;
use axum::extract::{Path, Query, State};
use clickhouse::{Client, Row};
use control_core::{ConfigMutationRecord, MachineCommandInvocation, MachineEvent, MachineIdentificationUnique, MachineMeasurementSnapshot, MachineStateMutationRecord, ScalarValue};

use crate::{SharedState, api::common::PropertyHistoryQuery};
use crate::api::common::{ApiError, get_machine_info, get_property_info};

type GetResponse = Vec<Record>;

#[derive(Serialize)]
pub(super) enum Record {
    ConfigChanged(ConfigMutationRecord),
    StateChange(MachineStateMutationRecord),
    MeasurementCaptured(MachineMeasurementSnapshot),
    EventEmitted(MachineEvent),
    CommandInvoked(MachineCommandInvocation)
}

pub(super) async fn get(
    State(state): State<Arc<SharedState>>,
    Path((slug, serial, name)): Path<(String, u16, String)>,
    Query(query): Query<PropertyHistoryQuery>,
) -> Result<Json<GetResponse>, ApiError> {
    // get schema info
    let schemas = state.schemas.load();
    let (ident, schema) = get_machine_info(&schemas, &slug, serial)?;



    Ok(axum::Json(GetResponse { 
        // TODO: add unit
        data: result 
    }))
}

async fn fetch_machine_events() {
    
}

async fn fetch_all(
    client: &Client,
    ident: MachineIdentificationUnique,
    name: &str,
    query: PropertyHistoryQuery,
) -> Result<Vec<Item>, String> {
    let sql = init_sql(&query)?;

    let mut q = client.query(&sql).bind(ident.to_u64()).bind(name);

    if let Some(from) = query.get_time_span()?.from {
        q = q.bind(dt_to_ch_datetime64_ms(from));
    }

    if let Some(to) = query.get_time_span()?.to {
        q = q.bind(dt_to_ch_datetime64_ms(to));
    }

    q = q.bind(query.get_limit());
    q.fetch_all::<Item>().await.map_err(|e| format!("{e}"))
}

fn init_sql(query: &PropertyHistoryQuery) -> Result<String, String> {
    let mut sql = match &query.get_aggregation()? {
        Some(aggregation) => format!(
            r#"
            SELECT
                toDateTime64(toStartOfInterval(timestamp, {}), 3) AS timestamp,
                value,
            FROM machine_measurements
            WHERE identity = ?
            AND name = ?
            "#,
            aggregation.interval.to_ch(),
        ),
        None => r#"
            SELECT timestamp, value
            FROM machine_measurements
            WHERE identity = ?
            AND name = ?
        "#.to_string(),
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
