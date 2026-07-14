use std::sync::Arc;
use axum::routing::get;
use axum::routing::post;
use tokio::net::TcpListener;
use crate::SharedState;

mod types;
pub use types::TransactionId;
pub use types::RuntimeRequest;
pub use types::RuntimeTransaction;

mod machines;
// mod machine;
mod config;
mod config_property;
mod state;
mod state_property;
mod measurements;
mod measurement_property;
mod machine_events;
mod machine_event;
mod commands;
mod command;
mod analytics;

pub(crate) async fn run(state: SharedState) -> anyhow::Result<()> {
    let api_address = &state.config.api_address;

    let router = axum::Router::new()
        // --- machines ---
        // route for listing all machines registered and whether they are connected or not
        .route("/api/v3/machines", get(machines::get))

        // // route for listing all properties, events and commands of a machine
        // .route("/api/v3/machines/{name}/{serial}", get(machine::get))

        // --- config ---
        // route for retrieving the current value of all config properties for a machine
        // .route(
        //     "/api/v3/machines/{slug}/{serial}/config",
        //     get(config::get),
        // )
        // route for reading a single config property
        .route(
            "/api/v3/machines/{slug}/{serial}/config/{property_name}",
            get(config_property::get),
        )
        // route for changing a config property
        .route(
            "/api/v3/machines/{name}/{serial}/config/{property_name}",
            post(config_property::post),
        )

        /*
        // --- state ---
        // route for retrieving the current value of all state properties for a machine
        .route(
            "/api/v3/machines/{name}/{serial}/state",
            get(state::handle),
        )
        // route for reading a single state property
        .route(
            "/api/v3/machines/{name}/{serial}/state/{property_name}",
            get(state_property::get),
        )

        // --- measurements ---
        // route for retrieving the current value of all measurements for a machine
        .route(
            "/api/v3/machines/{name}/{serial}/measurements",
            get(measurements::handle),
        )
        // route for reading a single measurement
        .route(
            "/api/v3/machines/{name}/{serial}/measurements/{property_name}",
            get(measurement_property::get),
        )

        // --- machines events ---
        // route for retrieving recent events for a machine
        .route(
            "/api/v3/machines/{name}/{serial}/events",
            get(machine_events::get),
        )
        // route for reading a single event by name/id
        .route(
            "/api/v3/machines/{name}/{serial}/events/{event_name}",
            get(machine_event::get),
        )

        // --- commands ---
        // route for listing available commands for a machine
        .route(
            "/api/v3/machines/{name}/{serial}/commands",
            get(commands::handle),
        )
        // route for issuing a command
        .route(
            "/api/v3/machines/{name}/{serial}/commands/{command_name}",
            post(command::post),
        )

        // --- misc ---
        // route for exporting analytics of a machine
        .route(
            "/api/v3/machines/{name}/{serial}/analytics",
            get(analytics::handle),
        )
    */

        // finish by including shared state
        .with_state(Arc::new(state.clone()));

    let listener = TcpListener::bind(&api_address).await?;
    println!("[RestApi] Listening on {}", &api_address);
    axum::serve(listener, router).await?;

    Ok(())
}