use std::sync::Arc;

use socketioxide::ParserConfig;
use socketioxide::extract::SocketRef;
use socketioxide::layer::SocketIoLayer;

pub async fn init_socketio(app_state: Arc<()>) -> SocketIoLayer {
    // create
    let (socketio_layer, io) = socketioxide::SocketIoBuilder::new()
        .max_buffer_size(1024)
        .with_parser(ParserConfig::msgpack())
        .build_layer();

    // Clone app_state for the first handler
    let app_state_main = app_state.clone();

    // set the on connect handler for main namespace
    io.ns("/main", move |socket: SocketRef| async move {
        handle_socket_connection(socket, app_state_main.clone());
    });

    // Clone app_state for the second handler
    let app_state_machine = app_state.clone();

    if let Err(err) = io.dyn_ns(
        "/machine/{vendor}/{machine}/{serial}",
        move |socket: SocketRef| async move {
            handle_socket_connection(socket, app_state_machine.clone());
        },
    ) {
        tracing::error!("Failed to detect machine namespace: {}", err);
    }

    // set the io to the app state
    let mut socketio_guard = app_state.socketio_setup.socketio.write().await;
    socketio_guard.replace(io);

    socketio_layer
}
