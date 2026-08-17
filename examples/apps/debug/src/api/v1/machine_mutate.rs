use std::fmt::Debug;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::machine::MachineSubscribeError;
use qitech_framework_core::report::{ConfigPropertyWriteError, ResourceAccessError};
use qitech_framework_core::request::{MachineExecuteCommandError, MachineSetConfigProperty, MachineUnsubscribeError, RuntimeRequestError};
use qitech_framework_hub::ActorContext;
use serde::Deserialize;
use serde_json::json;

use crate::LaserV1;
use crate::api::adapter;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub ident: MachineIdentificationUnique,
    pub data: serde_json::Value,
}

pub async fn post(
    State(ctx): State<ActorContext>,
    Json(body): Json<Request>,
) -> Response {
    let request = match body.ident.identification {
        LaserV1::IDENTIFICATION => match adapter::laser_v1::map_request(body.ident, body.data) {
            Ok(request) => request,
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_request",
                        "message": err.to_string(),
                    })),
                )
                    .into_response();
            }
        },

        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "machine_not_supported",
                    "message": "The requested machine is not supported",
                })),
            )
                .into_response();
        }
    };

    match ctx.send_request(request).await {
        Ok(Ok(())) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "status": "accepted",
            })),
        ).into_response(),

        Ok(Err(e)) => RuntimeRequestHttpError(e).into_response(),

        Err(err) => {
            tracing::error!(%err, "failed to send runtime request");

            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "runtime_unavailable",
                    "message": "No runtime is currently connected",
                }))
            ).into_response()
        }
    }
}

pub struct RuntimeRequestHttpError(pub RuntimeRequestError);

impl IntoResponse for RuntimeRequestHttpError {
    fn into_response(self) -> Response {
        fn error_response(
            status: StatusCode,
            error: &'static str,
            message: impl Into<String>,
        ) -> Response {
            (
                status,
                Json(json!({
                    "error": error,
                    "message": message.into(),
                })),
            )
                .into_response()
        }

        fn resource_access_response(error: ResourceAccessError) -> Response {
            match error {
                ResourceAccessError::MachineNotFound => error_response(
                    StatusCode::NOT_FOUND,
                    "machine_not_found",
                    "The requested machine was not found",
                ),

                ResourceAccessError::ResourceNotFound { kind, path } => error_response(
                    StatusCode::NOT_FOUND,
                    "resource_not_found",
                    format!("Resource of kind `{kind}` at `{path}` was not found"),
                ),

                ResourceAccessError::TypeMismatch { expected, actual } => error_response(
                    StatusCode::BAD_REQUEST,
                    "resource_type_mismatch",
                    format!(
                        "Expected resource type `{expected}`, but got `{actual}`"
                    ),
                ),
            }
        }

        match self.0 {
            RuntimeRequestError::WriteMachineDeviceInfo(error) => error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "machine_device_info_write_failed",
                error.to_string(),
            ),

            RuntimeRequestError::MachineSetConfigProperty(error) => match error {
                MachineSetConfigProperty::ResourceAccess(error) => {
                    resource_access_response(error)
                }

                MachineSetConfigProperty::WriteError(error) => match error {
                    ConfigPropertyWriteError::ValueTypeMismatch(err) => error_response(
                        StatusCode::BAD_REQUEST,
                        "config_property_value_type_mismatch",
                        err.to_string(),
                    ),

                    ConfigPropertyWriteError::NotWritable => error_response(
                        StatusCode::CONFLICT,
                        "config_property_not_writable",
                        "The requested configuration property is not writable",
                    ),

                    ConfigPropertyWriteError::ConstraintViolation(err) => error_response(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "config_property_constraint_violation",
                        err.to_string(),
                    ),
                },
            },

            RuntimeRequestError::MachineExecuteCommand(error) => match error {
                MachineExecuteCommandError::ResourceAccess(error) => {
                    resource_access_response(error)
                }

                MachineExecuteCommandError::ExecuteError(error) => error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "command_execution_failed",
                    error.to_string(),
                ),
            },

            RuntimeRequestError::MachineSubscribe(error) => match error {
                MachineSubscribeError::ResourceAccess(error) => {
                    resource_access_response(error)
                }

                MachineSubscribeError::ProviderNotFound => error_response(
                    StatusCode::NOT_FOUND,
                    "provider_not_found",
                    "The requested provider was not found",
                ),

                MachineSubscribeError::SubscriberNotFound => error_response(
                    StatusCode::NOT_FOUND,
                    "subscriber_not_found",
                    "The requested subscriber was not found",
                ),

                MachineSubscribeError::AlreadySubscribed => error_response(
                    StatusCode::CONFLICT,
                    "already_subscribed",
                    "The subscriber is already subscribed",
                ),

                MachineSubscribeError::UnsupportedMachine => error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "unsupported_machine",
                    "The machine does not support subscriptions",
                ),

                MachineSubscribeError::TooManySubscriptions => error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "too_many_subscriptions",
                    "The subscription limit has been reached",
                ),
            },

            RuntimeRequestError::MachineUnsubscribe(error) => match error {
                MachineUnsubscribeError::SubscriptionNotFound => error_response(
                    StatusCode::NOT_FOUND,
                    "subscription_not_found",
                    "The requested subscription was not found",
                ),
            },
        }
    }
}
