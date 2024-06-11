use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::Router;
use bevy::app::App;
use bevy::log::LogPlugin;
use bevy::prelude::info;
use bevy::MinimalPlugins;
use networking_server::MultiplayerSimpleNetServerPlugin;
use networking_shared::WEBSOCKETS_PORT;
use tower_http::services::ServeDir;

#[allow(clippy::expect_used)]
#[tokio::main]
async fn main() {
    let bevy_thread = std::thread::spawn(|| {
        run_bevy();
    });

    run_axum().await;

    bevy_thread.join().expect("Bevy thread panicked");
}

fn run_bevy() {
    let args: Vec<String> = std::env::args().collect();
    let address = match args.get(1).cloned() {
        None => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), WEBSOCKETS_PORT),
        Some(address_string) => {
            address_string
                .parse()
                .unwrap_or_else(|_| panic!("Unable to parse socket address {address_string}"))
        },
    };

    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(LogPlugin::default());
    app.add_plugins(MultiplayerSimpleNetServerPlugin { address });

    app.run();
}

#[allow(clippy::unwrap_used)]
async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn run_axum() {
    let router = Router::new().nest_service("/", ServeDir::new("static"));
    serve(router, 8080).await;
}
