use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};
use std::{io, net::SocketAddr};

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{MethodRouter, get, get_service};
use bevy::MinimalPlugins;
use bevy::app::App;
use bevy::log::LogPlugin;
use bevy::prelude::{info, trace};
use networking_server::MultiplayerSimpleNetServerPlugin;
use networking_server::metrics::PrometheusMetrics;
use networking_shared::PORT;
use tower_http::services::ServeDir;

#[derive(Default)]
enum ServeStatic {
    #[default]
    Local,
    Gcs,
}

async fn static_redirect_gcs(uri: Uri) -> impl IntoResponse {
    let path = uri.path();
    let gcs_url = format!("https://storage.googleapis.com/ts.krikis.online{path}");
    Redirect::temporary(&gcs_url)
}

const SERVE_STATIC_FROM_KEY: &str = "SERVE_STATIC_FROM";

async fn serve_metrics(State(metrics): State<PrometheusMetrics>) -> impl IntoResponse {
    metrics.render()
}

/// Depending on `SERVE_STATIC_FROM` environment variable, the server will serve static files from
/// either local or GCS storage.
#[tokio::main]
async fn main() {
    info!("Starting server on {PORT}...");
    let metrics = PrometheusMetrics::new();

    let serve_static = match std::env::var(SERVE_STATIC_FROM_KEY) {
        Ok(value) => {
            match value.as_str() {
                "gcs" => ServeStatic::Gcs,
                "local" => ServeStatic::Local,
                _ => ServeStatic::default(),
            }
        },
        Err(_) => ServeStatic::default(),
    };

    let router: Router<(), Body> = Router::new()
        .route("/health", get(health_check))
        .route("/liveness", get(liveness_check))
        .route("/metrics", get(serve_metrics).with_state(metrics.clone()));

    let serve_dir = ServeDir::new("static");
    let serve_static_local: MethodRouter<(), Body, Infallible> =
        get_service(serve_dir).handle_error(handle_error);

    let router = match serve_static {
        ServeStatic::Local => router,
        ServeStatic::Gcs => {
            router
                .route("/wasm-build/*path", get(static_redirect_gcs))
                .route("/assets/*path", get(static_redirect_gcs))
        },
    };

    let router = router.nest_service("/", serve_static_local);

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

    app.insert_resource(metrics);
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
