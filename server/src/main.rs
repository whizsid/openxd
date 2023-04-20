use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    convert::Infallible,
};

use app::App;
use config::{WS_HOST, WS_PORT, WS_PATH};
use futures::lock::Mutex;
use hyper::{Server, Body, Response};
use once_cell::sync::Lazy;
use routerify::{Router, RouterService};
use routerify_websocket::{WebSocket as RouterifyWebSocket, upgrade_ws};
use ws::WebSocket;

mod config;
mod ws;

static APP: Lazy<Mutex<App>> = Lazy::new(|| {
    Mutex::new(App::new())
});

async fn ws_handler(ws: RouterifyWebSocket) {
    println!("New websocket connection initialized");
    let local_ws = WebSocket::new(ws);
    let mut app = APP.lock().await;
    let mut session = app.create_session(local_ws);
    session.start().await;
}

fn router() -> Router<Body, Infallible> {

    // Create a router and specify the path and the handler for new websocket connections.
    Router::builder()
        // It will accept websocket connections at `/ws` path with any method type.
        .any_method(WS_PATH, upgrade_ws(ws_handler))
        // It will accept http connections at `/` path.
        .get("/", |_req| async move {
            Ok(Response::new("I also serve http requests".into()))
        })
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {

    let service = RouterService::new(router()).expect("Could not create router");
    let socket_addr = SocketAddr::new(
        IpAddr::from_str(WS_HOST).expect("Could not parse WS_HOST value as an IpAddr"),
        WS_PORT
            .parse()
            .expect("Could not parse WS_PORT value as a u16"),
    );

    let server = Server::bind(&socket_addr).serve(service);

    println!("App is running on: {}:{}", WS_HOST, WS_PORT);
    if let Err(err) = server.await {
        eprintln!("Server error: {:?}", err);
    }
}
