use std::{
    collections::BTreeMap,
    fmt::Debug,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
};

use app::{
    external::{
        create_project_using_existing_file, export_snapshot, get_current_tab_snapshot_id,
        CreateProjectUsingExistingFileError, GetCurrentTabSnapshotError,
    },
    App,
};
use config::{
    DB_NAME, DB_NAMESPACE, DB_PASSWORD, DB_URL, DB_USER, JWT_SECRET, WS_HOST, WS_PATH, WS_PORT,
};
use error::{AuthError, CreateProjectError, Error, SnapshotDownloadError, WebSocketOpenError};
use futures::{lock::Mutex, ready, TryStreamExt};
use hmac::{Hmac, Mac};
use hyper::{header::CONTENT_TYPE, Body, Request, Response, Server, StatusCode};
use jwt::VerifyWithKey;
use model::{SnapshotDownload, Ticket};
use multer::Multipart;
use querystring::querify;
use routerify::{prelude::RequestExt, Middleware, RouteError, Router, RouterService};
use routerify_cors::enable_cors_all;
use routerify_websocket::{upgrade_ws, WebSocket as RouterifyWebSocket};
use serde::Serialize;
use sha2::Sha256;
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
use surrealdb::Surreal;

use once_cell::sync::OnceCell;
use serde_json::ser::to_string;
use tokio::{io::AsyncWrite, sync::OnceCell as TokioOnceCell};
use tokio_util::io::StreamReader;
use transport::ReceiveError;
use ws::WebSocket;

mod config;
mod error;
mod model;
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

fn router() -> Router<Body, Error<StorageError>> {
    // Create a router and specify the path and the handler for new websocket connections.
    Router::builder()
        // It will accept websocket connections at `/ws` path with GET method type.
        .get(WS_PATH, ws_open_handler)
        .middleware(enable_cors_all())
        .middleware(api_auth(&[
            "/api/create-project",
            "/api/current-tab-snapshot",
        ]))
        .post("/api/create-project", oxd_upload_handler)
        .get("/api/current-tab-snapshot", current_tab_snapshot_handler)
        .get("/api/snapshot/:downloadId", download_snapshot_handler)
        .get("/", |_req| async move {
            Ok(Response::new("I also serve http requests".into()))
        })
        .err_handler(error_handler)
        .build()
        .unwrap()
}

async fn error_handler(route_err: RouteError) -> Response<Body> {
    let err: Box<Error<StorageError>> = route_err.downcast().unwrap();
    log::error!("{:?}", &err);
    let mut status_code: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;
    let mut err_code = "INTERNAL SERVER ERROR";
    match err.as_ref() {
        Error::Auth(_) => {
            status_code = StatusCode::UNAUTHORIZED;
            err_code = "UNAUTHORIZED";
        }
        Error::CreateProject(create_project_err) => match create_project_err {
            CreateProjectError::Inner(create_project_internal_err) => {
                match create_project_internal_err {
                    CreateProjectUsingExistingFileError::Utf8(_)
                    | CreateProjectUsingExistingFileError::Serde(_)
                    | CreateProjectUsingExistingFileError::UnsupportedFile
                    | CreateProjectUsingExistingFileError::UnsupportedAsset { path: _ } => {
                        status_code = StatusCode::BAD_REQUEST;
                        err_code = "FILE NOT VALID";
                    }
                    _ => {}
                }
            }
            _ => {
                status_code = StatusCode::BAD_REQUEST;
                err_code = "VALIDATION ERROR";
            }
        },
        Error::Multer(_) => {
            status_code = StatusCode::BAD_REQUEST;
            err_code = "MULTIPART REQUEST BODY INVALID";
        }
        Error::WebSocketOpen(ws_open_err) => match ws_open_err {
            WebSocketOpenError::PendingTicket {
                ticket_id: _,
                opened_at: _,
            } => {
                status_code = StatusCode::CONFLICT;
                err_code = "TICKET STILL PENDING";
            }
            WebSocketOpenError::ExpiredTicket {
                ticket_id: _,
                closed_at: _,
            } => {
                status_code = StatusCode::BAD_REQUEST;
                err_code = "EXPIRED TICKET";
            }
            WebSocketOpenError::TicketIdNotProvided | WebSocketOpenError::TryFromSlice(_) => {
                status_code = StatusCode::BAD_REQUEST;
                err_code = "TICKET ID VALIDATION FAILED";
            }
            WebSocketOpenError::TicketNotFound { ticket_id: _ } => {
                status_code = StatusCode::NOT_FOUND;
                err_code = "TICKET NOT FOUND";
            }
        },
        Error::CurrentSnapshot(current_snapshot_err) => match current_snapshot_err {
            GetCurrentTabSnapshotError::NoActiveSession
            | GetCurrentTabSnapshotError::NoTabOpened => {
                status_code = StatusCode::BAD_REQUEST;
                err_code = "NO TAB OR SESSION CREATED";
            }
            _ => {}
        },
        Error::SnapshotDownload(dwnld_err) => match dwnld_err {
            SnapshotDownloadError::Invalid { download_id: _ }
            | SnapshotDownloadError::AlreadyDownloaded { download_id: _ } => {
                status_code = StatusCode::NOT_FOUND;
                err_code = "NOT FOUND";
            }
            _ => {}
        },
        _ => {}
    }

    return Response::builder()
        .status(status_code)
        .body(Body::from(err_code))
        .unwrap();
}

#[derive(Clone, Debug)]
pub struct UserId(pub String);

pub fn api_auth(routes: &'static [&str]) -> Middleware<hyper::Body, Error<StorageError>> {
    Middleware::pre(move |req| async move {
        let uri = req.uri();
        if routes.contains(&&uri.to_string().as_str()) {
            match req.headers().get(hyper::header::AUTHORIZATION) {
                Some(auth_head) => {
                    let auth_head_str = auth_head
                        .to_str()
                        .map_err(|e| Error::Auth(AuthError::HeaderToStr(e)))?;
                    let mut auth_head_split = auth_head_str.split(' ').into_iter();
                    if let Some(bearer) = auth_head_split.next() {
                        if bearer.to_lowercase() == "bearer" {
                            match auth_head_split.next() {
                                Some(token) => {
                                    let key: Hmac<Sha256> = Hmac::new_from_slice(
                                        JWT_SECRET.as_bytes(),
                                    )
                                    .map_err(|e| Error::Auth(AuthError::InvalidLength(e)))?;
                                    let claims: BTreeMap<String, String> = token
                                        .verify_with_key(&key)
                                        .map_err(|e| Error::Auth(AuthError::Jwt(e)))?;
                                    let user_id = claims["sub"].clone();
                                    req.set_context(UserId(user_id));
                                    Ok(req)
                                }
                                None => Err(Error::Auth(AuthError::BearerNotProvided)),
                            }
                        } else {
                            Err(Error::Auth(AuthError::NotBearer))
                        }
                    } else {
                        Err(Error::Auth(AuthError::HeaderNotProvided))
                    }
                }
                None => Err(Error::Auth(AuthError::HeaderNotProvided)),
            }
        } else {
            Ok(req)
        }
    })
}

/// Handling the web socket connections
pub async fn ws_open_handler(req: Request<Body>) -> Result<Response<Body>, Error<StorageError>> {
    if let Some(query_str) = req.uri().query() {
        let parsed_query = querify(query_str);
        let mut query_iter = parsed_query.iter().filter(|q| q.0 == "ticket");
        let token = query_iter.next();
        if let Some((_, ticket)) = token {
            let app = get_app().await;
            let app = app.lock().await;
            let ticket_opt: Option<Ticket> =
                app.database().select((Ticket::TABLE, *ticket)).await?;

            match ticket_opt {
                Some(ticket) => {
                    let opened_at = ticket.opened_at.clone();
                    if opened_at.is_some() && !ticket.allow_connect_again {
                        return Err(Error::WebSocketOpen(
                            error::WebSocketOpenError::ExpiredTicket {
                                ticket_id: ticket.id.to_string(),
                                closed_at: ticket.closed_at.unwrap(),
                            },
                        ));
                    }

                    if let Some(opened_at) = ticket.opened_at {
                        if let Some(closed_at) = ticket.closed_at {
                            if opened_at >= closed_at {
                                return Err(Error::WebSocketOpen(
                                    error::WebSocketOpenError::PendingTicket {
                                        ticket_id: ticket.id.to_string(),
                                        opened_at,
                                    },
                                ));
                            }
                        } else {
                            return Err(Error::WebSocketOpen(
                                error::WebSocketOpenError::PendingTicket {
                                    ticket_id: ticket.id.to_string(),
                                    opened_at,
                                },
                            ));
                        }
                    }

                    // Converting to raw bytes. Because string not implements copy
                    let user_id_str = ticket.user.id.to_raw();
                    let user_id_bytes: [u8; 20] = user_id_str
                        .as_bytes()
                        .try_into()
                        .map_err(|e| Error::WebSocketOpen(WebSocketOpenError::TryFromSlice(e)))?;

                    let ws_handler = move |ws: RouterifyWebSocket| async move {
                        let user_id = String::from_utf8(Vec::from(user_id_bytes)).unwrap();
                        let local_ws = WebSocket::new(ws);
                        let app = get_app().await;
                        let mut app = app.lock().await;
                        let mut session = app
                            .create_session(user_id, local_ws, get_storage().clone())
                            .await
                            .unwrap();

                        loop {
                            let message = session.receive_message().await;
                            match message {
                                Ok(message) => {
                                    session.handle_message(message).await;
                                }
                                Err(e) => match e {
                                    ReceiveError::Terminated => {
                                        session.close().await;
                                        break;
                                    }
                                    _ => {}
                                },
                            }
                        }
                    };

                    return upgrade_ws(ws_handler)(req).await;
                }
                None => {
                    return Err(Error::WebSocketOpen(
                        error::WebSocketOpenError::TicketNotFound {
                            ticket_id: String::from(*ticket),
                        },
                    ));
                }
            }
        }
    }

    return Err(Error::WebSocketOpen(
        error::WebSocketOpenError::TicketIdNotProvided,
    ));
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
pub async fn oxd_upload_handler(req: Request<Body>) -> Result<Response<Body>, Error<StorageError>> {
    let user_id = req.context::<UserId>().unwrap();
    let boundary = req
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| multer::parse_boundary(ct).ok());

    if boundary.is_none() {
        return Err(Error::CreateProject(
            error::CreateProjectError::BoundaryNotProvided,
        ));
    }

    let mut multipart = Multipart::new(req.into_body(), boundary.unwrap());
    let mut project_name: Option<String> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(field_name) = field.name() {
            if field_name == "project_name" {
                let project_name_val = field.text().await?;
                project_name = Some(project_name_val);
            } else if field_name == "file" {
                if let Some(project_name) = project_name {
                    let stream_reader = StreamReader::new(
                        field.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
                    );
                    let project = create_project_using_existing_file(
                        get_db().await.clone(),
                        get_storage().clone(),
                        stream_reader,
                        project_name,
                        user_id.0.clone(),
                    )
                    .await
                    .map_err(|e| Error::CreateProject(CreateProjectError::Inner(e)))?;
                    let project_id_str = project.id.to_string();
                    let success_response = OxdUploadSuccessResponse::new(project_id_str);
                    let json_content = to_string(&success_response).unwrap();
                    return Ok(Response::builder()
                        .status(StatusCode::CREATED)
                        .body(Body::from(json_content))
                        .unwrap());
                } else {
                    return Err(Error::CreateProject(
                        CreateProjectError::ProjectNameNotProvided,
                    ));
                }
            }
        }
    }

    return Err(Error::CreateProject(CreateProjectError::FileNotProvided));
}

pub struct SenderWriter {
    sender: hyper::body::Sender,
}

impl SenderWriter {
    pub fn new(sender: hyper::body::Sender) -> SenderWriter {
        SenderWriter { sender }
    }
}

impl AsyncWrite for SenderWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        ready!(self
            .sender
            .poll_ready(cx)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?);

        match self.sender.try_send_data(Box::<[u8]>::from(buf).into()) {
            Ok(()) => Poll::Ready(Ok(buf.len())),
            Err(_) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Body closed",
            ))),
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let res = self.sender.poll_ready(cx);
        res.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.poll_flush(cx)
    }
}

#[derive(Serialize)]
pub struct CurrentTabSnapshotResponse {
    download_id: String,
}

impl CurrentTabSnapshotResponse {
    pub fn new(download_id: String) -> CurrentTabSnapshotResponse {
        CurrentTabSnapshotResponse { download_id }
    }
}

pub async fn current_tab_snapshot_handler(
    req: Request<Body>,
) -> Result<Response<Body>, Error<StorageError>> {
    let db = get_db().await;
    let user_id = req.context::<UserId>().unwrap();

    let snapshot_id = get_current_tab_snapshot_id(db.clone(), user_id.0.clone()).await?;
    let snapshot_download = SnapshotDownload::new(snapshot_id, user_id.0);
    let created_download: SnapshotDownload = db
        .create(SnapshotDownload::TABLE)
        .content(snapshot_download)
        .await?;

    let success_response = CurrentTabSnapshotResponse::new(created_download.id.to_string());
    let json_content = to_string(&success_response).unwrap();
    return Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(json_content))
        .unwrap());
}

pub async fn download_snapshot_handler(
    req: Request<Body>,
) -> Result<Response<Body>, Error<StorageError>> {
    let db = get_db().await;
    let download_id = req.param("downloadId").unwrap();

    let snapshot_download_opt: Option<SnapshotDownload> = db
        .select((SnapshotDownload::TABLE, download_id.clone()))
        .await?;

    if let Some(snapshot_download) = snapshot_download_opt {
        if snapshot_download.downloaded_at.is_some() {
            return Err(Error::SnapshotDownload(
                SnapshotDownloadError::AlreadyDownloaded {
                    download_id: download_id.clone(),
                },
            ));
        }

        let mut snapshot_download_cpy = snapshot_download.clone();
        snapshot_download_cpy.mark_as_downloaded();

        db.update((SnapshotDownload::TABLE, snapshot_download_cpy.id.clone()))
            .content(snapshot_download_cpy)
            .await?;

        let snapshot_id = snapshot_download.snapshot.id.to_string();

        let (sender, body) = Body::channel();
        let sender_writer = SenderWriter::new(sender);
        tokio::spawn(async {
            export_snapshot(
                get_db().await.clone(),
                get_storage().clone(),
                sender_writer,
                snapshot_id,
            )
            .await
            .unwrap();
        });
        Ok(Response::builder()
            .header("Content-Type", "application/openxd")
            .body(body)
            .unwrap())
    } else {
        return Err(Error::SnapshotDownload(SnapshotDownloadError::Invalid {
            download_id: download_id.clone(),
        }));
    }
}
