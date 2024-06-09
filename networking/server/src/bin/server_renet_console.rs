use axum::{response::Html, routing::get, Router};
use bevy::log::LogPlugin;
use bevy::prelude::App;
use bevy::MinimalPlugins;
use networking_renet_server::server::networking::MultiplayerRenetServerPlugin;
use networking_renet_shared::parse_server_address;

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
    let address_string: Option<String> = args.get(1).cloned();
    let address = parse_server_address(address_string.clone())
        .unwrap_or_else(|_| panic!("Failed to parse server address {address_string:?}"));

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(LogPlugin::default());
    app.add_plugins(MultiplayerRenetServerPlugin::new(address));
    app.run();
}

#[allow(clippy::unwrap_used)]
async fn run_axum() {
    let app = Router::new().route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello from Axum!</h1>")
}
