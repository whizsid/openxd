use std::{
    convert::Infallible,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use app::{
    external::{create_project_using_existing_file, CreateProjectUsingExistingFileError},
    App,
};
use config::{DB_NAME, DB_NAMESPACE, DB_PASSWORD, DB_URL, DB_USER, WS_HOST, WS_PATH, WS_PORT};
use futures::{lock::Mutex, TryStreamExt};
use hyper::{header::CONTENT_TYPE, Body, Request, Response, Server, StatusCode};
use multer::Multipart;
use querystring::querify;
use routerify::{Router, RouterService};
use routerify_cors::enable_cors_all;
use routerify_websocket::{upgrade_ws, WebSocket as RouterifyWebSocket};
use serde::{Deserialize, Serialize};
use storage::StorageImpl;

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
    sql::{Datetime, Id, Thing},
    Surreal,
};

use once_cell::sync::OnceCell;
use serde_json::ser::to_string;
use tokio::sync::OnceCell as TokioOnceCell;
use tokio_util::io::StreamReader;
use ws::WebSocket;

mod config;
mod storage;
mod ws;

static STORAGE: OnceCell<Arc<StorageImpl>> = OnceCell::new();

pub fn get_storage() -> &'static Arc<StorageImpl> {
    STORAGE.get_or_init(|| Arc::new(StorageImpl))
}

static DB: TokioOnceCell<Arc<Surreal<DbClient>>> = TokioOnceCell::const_new();

pub async fn get_db() -> &'static Arc<Surreal<DbClient>> {
    DB.get_or_init(|| async {
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
        Arc::new(db)
    })
    .await
}

static APP: TokioOnceCell<Mutex<App<DbClient>>> = TokioOnceCell::const_new();

pub async fn get_app() -> &'static Mutex<App<DbClient>> {
    APP.get_or_init(|| async {
        let db = get_db().await;
        Mutex::new(App::new(db.clone()))
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
                            let user_id = String::from_utf8(Vec::from(user_id_bytes)).unwrap();
                            let local_ws = WebSocket::new(ws);
                            let app = get_app().await;
                            let mut app = app.lock().await;
                            let mut session = app.create_session(user_id, local_ws).await.unwrap();
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
                let project_res = create_project_using_existing_file(
                    get_db().await.clone(),
                    get_storage().clone(),
                    stream_reader,
                )
                .await;
                return match project_res {
                    Ok(project) => {
                        let project_id_str = project.id.to_string();
                        let success_response = OxdUploadSuccessResponse::new(project_id_str);
                        let json_content = to_string(&success_response).unwrap();
                        Ok(Response::builder()
                            .status(StatusCode::CREATED)
                            .body(Body::from(json_content))
                            .unwrap())
                    }
                    Err(err) => match err {
                        CreateProjectUsingExistingFileError::Io(_) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from("IO ERROR"))
                            .unwrap()),
                        CreateProjectUsingExistingFileError::Db(_) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from("DB ERROR"))
                            .unwrap()),
                        CreateProjectUsingExistingFileError::Storage(_) => Ok(Response::builder()
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
