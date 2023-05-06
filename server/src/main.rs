use std::{
    convert::Infallible,
    net::{IpAddr, SocketAddr},
    str::FromStr, fmt::Debug,
};

use app::{App, cache::CacheFileError};
use config::{WS_HOST, WS_PATH, WS_PORT};
use futures::{future::{ok, Ready}, lock::Mutex};
use hyper::{header::CONTENT_TYPE, Body, Request, Response, Server, StatusCode};
use multer::Multipart;
use once_cell::sync::Lazy;
use querystring::querify;
use routerify::{Router, RouterService};
use routerify_cors::enable_cors_all;
use routerify_websocket::{upgrade_ws, WebSocket as RouterifyWebSocket};
use ws::WebSocket;
use tokio_util::io::StreamReader;

mod config;
mod ws;

static APP: Lazy<Mutex<App<u8, u8>>> = Lazy::new(|| Mutex::new(App::new()));

fn router() -> Router<Body, Infallible> {
    // Create a router and specify the path and the handler for new websocket connections.
    Router::builder()
        // It will accept websocket connections at `/ws` path with GET method type.
        .get(WS_PATH, ws_open_handler)
        .middleware(enable_cors_all())
        .post("/cache", oxd_upload_handler)
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

#[derive(Debug)]
pub struct StorageError;

pub enum OxdUploadError {
    Hyper(hyper::Error),
    Multer(multer::Error),
    CacheFile(CacheFileError<StorageError>),
}

pub fn ws_open_handler<E: Debug>(req: Request<Body>) -> Ready<Result<Response<Body>, E>> {
    if let Some(query_str) = req.uri().query() {
                let parsed_query = querify(query_str);
                let mut query_iter = parsed_query.iter().filter(|q| q.0 == "ticket");
                let token = query_iter.next();
                if let Some(_token) = token {
                    let user_id = 1;

                    let ws_handler = move |ws: RouterifyWebSocket| async move {
                        println!("New websocket connection initialized {}", user_id);
                        let local_ws = WebSocket::new(ws);
                        let mut app = APP.lock().await;
                        let mut session = app.create_session(local_ws);
                        session.start().await;
                    };

                    return upgrade_ws(ws_handler)(req);
                }
            }

            let mut response = Response::new("Unauthorized".into());
            let status = response.status_mut();
            (*status) = StatusCode::UNAUTHORIZED;

            ok(response)
}

pub async fn oxd_upload_handler(req: Request<Body>) -> Result<Response<Body>, OxdUploadError> {
    let boundary = req
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| multer::parse_boundary(ct).ok());

    if boundary.is_none() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("BAD REQUEST"))
            .unwrap());
    }

    let mut multipart = Multipart::new(req.into_body(), boundary.unwrap());
    while let Some(field) = multipart.next_field().await? {
        if let Some(field_name) = field.name() {
            if field_name == "file" {
                let mut stream_reader = StreamReader::new(field);
                let mut app = APP.lock().await;
                let session_id = app.create_session_with_file(stream_reader).await;
            }
        }
    }

    let body = req.into_body();
    let mut app = APP.lock().await;
    app.create_session_with_file(&[1, 2, 3])?;
    Ok(Response::new("Test".into()))
}
