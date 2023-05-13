use std::{
    convert::Infallible,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use app::{cache::CacheFileError, App};
use config::{DB_NAME, DB_NAMESPACE, DB_PASSWORD, DB_URL, DB_USER, WS_HOST, WS_PATH, WS_PORT};
use futures::{
    future::{ok, Ready},
    lock::Mutex,
    TryStreamExt,
};
use hyper::{header::CONTENT_TYPE, Body, Request, Response, Server, StatusCode};
use multer::Multipart;
use querystring::querify;
use routerify::{Router, RouterService};
use routerify_cors::enable_cors_all;
use routerify_websocket::{upgrade_ws, WebSocket as RouterifyWebSocket};
use serde::{Deserialize, Serialize};
use storage::{StorageError, StorageImpl};

#[cfg(any(feature = "db-http", feature = "db-https"))]
use surrealdb::engine::remote::http::Client as DbClient;
#[cfg(feature = "db-http")]
use surrealdb::engine::remote::http::Http as DbConnection;
#[cfg(feature = "db-https")]
use surrealdb::engine::remote::http::Https as DbConnection;
#[cfg(any(feature = "db-ws", feature = "db-wss"))]
use surrealdb::engine::remote::ws::Client as DbClient;
#[cfg(feature = "db-ws")]
use surrealdb::engine::remote::ws::Ws as DbConnection;
#[cfg(feature = "db-wss")]
use surrealdb::engine::remote::ws::Wss as DbConnection;
#[cfg(feature = "db-auth-database")]
use surrealdb::opt::auth::Database as DatabaseAuth;
#[cfg(feature = "db-auth-namespace")]
use surrealdb::opt::auth::Namespace as NamespaceAuth;
#[cfg(feature = "db-auth-root")]
use surrealdb::opt::auth::Root as RootAuth;
use surrealdb::{
    sql::Id,
    sql::{Datetime, Thing},
    Surreal,
};

use serde_json::ser::to_string;
use tokio::sync::OnceCell;
use tokio_util::io::StreamReader;
use ws::WebSocket;

mod config;
mod storage;
mod ws;

static APP: OnceCell<Mutex<App<PathBuf, StorageError, DbClient, StorageImpl>>> =
    OnceCell::const_new();

async fn get_app() -> &'static Mutex<App<PathBuf, StorageError, DbClient, StorageImpl>> {
    APP.get_or_init(|| async {
        let storage = StorageImpl;
        let db = Surreal::new::<DbConnection>(DB_URL).await.unwrap();
        db.use_ns(DB_NAMESPACE).use_db(DB_NAME).await.unwrap();
        #[cfg(feature = "db-auth-root")]
        db.signin(RootAuth {
            username: DB_USER,
            password: DB_PASSWORD,
        })
        .await
        .unwrap();
        #[cfg(feature = "db-auth-database")]
        db.signin(DatabaseAuth {
            username: DB_USER,
            password: DB_PASSWORD,
            namespace: DB_NAMESPACE,
            database: DB_NAME,
        })
        .await
        .unwrap();
        #[cfg(feature = "db-auth-namespace")]
        db.signin(NamespaceAuth {
            username: DB_URL,
            password: DB_PASSWORD,
            namespace: DB_NAMESPACE,
        })
        .await
        .unwrap();
        Mutex::new(App::new(db, storage))
    })
    .await
}

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

#[derive(Serialize, Deserialize)]
pub struct Ticket {
    id: Id,
    created_at: Datetime,
    opened_at: Option<Datetime>,
    closed_at: Option<Datetime>,
    exited_at: Option<Datetime>,
    allow_connect_again: bool,
    user: Thing,
}

/// Handling the web socket connections
pub async fn ws_open_handler<E: std::error::Error + Send + Sync + 'static>(
    req: Request<Body>,
) -> Result<Response<Body>, E> {
    if let Some(query_str) = req.uri().query() {
        let parsed_query = querify(query_str);
        let mut query_iter = parsed_query.iter().filter(|q| q.0 == "ticket");
        let token = query_iter.next();
        if let Some((_, ticket)) = token {
            let app = get_app().await;
            let app = app.lock().await;
            let ticket_res: Result<Option<Ticket>, _> =
                app.database().select(("ticket", *ticket)).await;

            match ticket_res {
                Ok(ticket_opt) => match ticket_opt {
                    Some(ticket) => {
                        if ticket.opened_at.is_some() && !ticket.allow_connect_again {
                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from("EXPIRED TICKET"))
                                .unwrap());
                        }

                        if let Some(opened_at) = ticket.opened_at {
                            if let Some(closed_at) = ticket.closed_at {
                                if opened_at >= closed_at {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from("ALREADY OPENED"))
                                        .unwrap());
                                }
                            } else {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from("ALREADY OPENED"))
                                    .unwrap());
                            }
                        }

                        // Converting to raw bytes. Because string not implements copy
                        let user_id_str = ticket.user.id.to_raw();
                        let user_id_bytes: [u8; 20] = user_id_str.as_bytes().try_into().unwrap();

                        let ws_handler = move |ws: RouterifyWebSocket| async move {
                            let i_am_using_user_id = user_id_bytes;
                            let local_ws = WebSocket::new(ws);
                            let app = get_app().await;
                            let mut app = app.lock().await;
                            let mut session = app.init_session(local_ws);
                            session.start().await;
                        };

                        return upgrade_ws(ws_handler)(req).await;
                    }
                    None => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from("INVALID TICKET"))
                            .unwrap());
                    }
                },

                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("DB ERROR"))
                        .unwrap());
                }
            }
        }
    }

    let mut response: Response<Body> = Response::new("Unauthorized".into());
    let status = response.status_mut();
    (*status) = StatusCode::UNAUTHORIZED;

    Ok(response)
}

#[derive(Serialize)]
pub struct OxdUploadSuccessResponse {
    id: String,
}

impl OxdUploadSuccessResponse {
    pub fn new(session_id: String) -> OxdUploadSuccessResponse {
        OxdUploadSuccessResponse { id: session_id }
    }
}

/// Handling the oxd file uploads
pub async fn oxd_upload_handler<E: std::error::Error + Send + Sync>(
    req: Request<Body>,
) -> Result<Response<Body>, E> {
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
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(field_name) = field.name() {
            if field_name == "file" {
                let stream_reader = StreamReader::new(
                    field.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
                );
                let app = get_app().await;
                let app = app.lock().await;
                let session_id_res = app.create_session_with_file(stream_reader).await;
                return match session_id_res {
                    Ok(session_id) => {
                        let session_id_str = session_id.to_string();
                        let success_response = OxdUploadSuccessResponse::new(session_id_str);
                        let json_content = to_string(&success_response).unwrap();
                        Ok(Response::builder()
                            .status(StatusCode::CREATED)
                            .body(Body::from(json_content))
                            .unwrap())
                    }
                    Err(err) => match err {
                        CacheFileError::Io(_) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from("IO ERROR"))
                            .unwrap()),
                        CacheFileError::Db(_) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from("DB ERROR"))
                            .unwrap()),
                        CacheFileError::Storage(_) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from("STORAGE ERROR"))
                            .unwrap()),
                        _ => Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from("INVALID FILE"))
                            .unwrap()),
                    },
                };
            }
        }
    }

    return Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("BAD REQUEST"))
        .unwrap());
}
