use std::{collections::BTreeMap, sync::Arc};

use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use chrono::{DateTime, Utc};
use control_core::{MachineIdentification, MachineIdentificationUnique, schema::v1_0::{MachineSchema, Property, PropertyKind}};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{api::types::{Aggregation, AggregationOperation, Interval, Ordering, TimeSpan}};

const LIMIT_DEFAULT: u64 = 100;
const LIMIT_MAXIMUM: u64 = 1_000_000;

#[derive(Deserialize)]
pub struct PropertyHistoryQuery {
    /// Start timestamp.
    #[serde(default, with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub from: Option<DateTime<Utc>>,

    /// End timestamp.
    #[serde(default, with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub to: Option<DateTime<Utc>>,

    /// Read the last x duration (s, m, d, h supported).
    pub last: Option<Interval>,

    /// Interval for aggregation
    pub interval: Option<Interval>,

    /// Aggregation method
    #[serde(rename = "aggregate")]
    pub operation: Option<AggregationOperation>,

    /// Result ordering
    pub ordering: Option<Ordering>,

    /// Maximum number of samples to return.
    pub limit: Option<u64>,

    // /// Response format.
    // pub format: Option<ResponseFormat>,
}

impl PropertyHistoryQuery {
    pub fn get_time_span(
        &self,
    ) -> Result<TimeSpan, String> {
        match (self.last.clone(), self.from, self.to) {
            // last only
            (Some(last), None, None) => {
                let duration = last.to_duration();
                Ok(TimeSpan::new(Some(Utc::now() - duration), None))
            }
            // from and to
            (None, Some(from), Some(to)) => Ok(TimeSpan::new(Some(from), Some(to))),
            // from only
            (None, Some(from), None) => Ok(TimeSpan::new(Some(from), None)),
            // to only
            (None, None, Some(to)) => Ok(TimeSpan::new(None, Some(to))) ,
            // nothing
            (None, None, None) => Ok(TimeSpan::new(None, None)),
            // invalid combinations
            (Some(_), _, _) => {
                Err("'last' cannot be combined with 'from' or 'to'".into())
            }
        }
    }

    pub fn get_aggregation(&self) -> Result<Option<Aggregation>, String> {
        match (self.operation, self.interval.clone()) {
            // missing interval
            (Some(_), None) => Err("'aggregate' requires 'interval'".into()),
            // missing aggregate
            (None, Some(_)) => {
                Err("'interval' requires 'aggregate'".into())
            }
            // ok
            (Some(operation), Some(interval)) => {
                Ok(Some(Aggregation { operation, interval }))
            },
            // no aggregate requested
            _ => Ok(None),
        }
    }

    pub fn get_ordering(&self) -> Ordering {
        self.ordering.unwrap_or(Ordering::Ascending)
    }

    pub fn get_limit(&self) -> u64 {
        self.limit.unwrap_or(LIMIT_DEFAULT).min(LIMIT_MAXIMUM)
    }
}

pub fn get_machine_info<'a>(
    schemas: &'a arc_swap::Guard<Arc<BTreeMap<MachineIdentification, MachineSchema>>>,
    slug: &str,
    serial: u16,
) -> Result<(MachineIdentificationUnique, &'a MachineSchema), ApiError> {
    let Some((ident, schema)) = schemas.iter().find(|(_, s)| s.name == slug) else {
        return Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: format!("machine '{slug}' not found"),
        });
    };

    let ident_unique = MachineIdentificationUnique {
        vendor: ident.vendor,
        machine: ident.machine,
        serial,
    };

    Ok((ident_unique, schema))
}

pub fn get_property_info<'a, T>(
    items: &'a IndexMap<String, Property<T>>,
    name: &str,
) -> Result<&'a T, ApiError> {
    let path: Vec<&str> = name.split('.').collect();

    let mut current = items;

    for (index, part) in path.iter().enumerate() {
        let Some(prop) = current.get(*part) else {
            return Err(ApiError {
                status: StatusCode::NOT_FOUND,
                message: format!("property '{name}' not found"),
            });
        };

        if index == path.len() - 1 {
            return match &prop.kind {
                PropertyKind::Value(value) => Ok(value),

                PropertyKind::Group(_) => Err(ApiError {
                    status: StatusCode::BAD_REQUEST,
                    message: format!("property '{name}' is a group"),
                }),
            };
        }

        let PropertyKind::Group(children) = &prop.kind else {
            return Err(ApiError {
                status: StatusCode::NOT_FOUND,
                message: format!("property '{name}' not found"),
            });
        };

        current = children;
    }

    Err(ApiError {
        status: StatusCode::NOT_FOUND,
        message: format!("property '{name}' not found"),
    })
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        ).into_response()
    }
}
