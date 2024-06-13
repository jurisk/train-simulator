use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};
use std::{io, net::SocketAddr};

use axum::body::Body;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, get_service, MethodRouter};
use axum::Router;
use bevy::app::App;
use bevy::log::LogPlugin;
use bevy::prelude::{info, trace};
use bevy::MinimalPlugins;
use networking_server::MultiplayerSimpleNetServerPlugin;
use networking_shared::PORT;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    info!("Starting server on {PORT}...");

    let serve_dir = ServeDir::new("static");
    let serve_dir_static: MethodRouter<(), Body, Infallible> =
        get_service(serve_dir).handle_error(handle_error);
    let router: Router<(), Body> = Router::new()
        .nest_service("/", serve_dir_static)
        .route("/health", get(health_check))
        .route("/liveness", get(liveness_check));

    let args: Vec<String> = std::env::args().collect();
    let address = match args.get(1).cloned() {
        None => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), PORT),
        Some(address_string) => {
            address_string
                .parse()
                .unwrap_or_else(|_| panic!("Unable to parse socket address {address_string}"))
        },
    };

    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(LogPlugin::default());
    app.add_plugins(MultiplayerSimpleNetServerPlugin {
        router: Arc::new(Mutex::new(router)),
        address,
    });

    app.run();
}

async fn health_check() -> impl IntoResponse {
    trace!("Health check OK");
    "OK"
}

async fn liveness_check() -> impl IntoResponse {
    trace!("Liveness check OK");
    "OK"
}

async fn handle_error(err: io::Error) -> impl IntoResponse {
    trace!("Error: {err:?}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
