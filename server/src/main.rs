use std::net::SocketAddr;

use app::Session;
use config::{WS_HOST, WS_PORT};
use tokio::net::{TcpListener, TcpStream};
use ws::WebSocket;

mod config;
mod ws;

async fn handle_connection(raw_stream: TcpStream, _addr: SocketAddr) {
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("Connection Established");

    let websocket = WebSocket::new(ws_stream);
    let mut session = Session::new(websocket);
    session.start().await;
}

#[tokio::main]
async fn main() {
    let ws_url = format!("{}:{}", WS_HOST, WS_PORT);

    let try_socket = TcpListener::bind(ws_url).await;
    let listener = try_socket.expect(&format!(
        "Can not bind the websocket to {}:{}",
        WS_HOST, WS_PORT
    ));

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}
