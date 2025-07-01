use std::{convert::Infallible, fs, path::PathBuf, sync::Arc };
use hyper::{
    body::{Bytes, Incoming}, header::{HeaderValue, CONTENT_TYPE}, server::conn::http1, service::service_fn, Request, Response, StatusCode
};
use http_body_util::{Full, BodyExt};
use tokio::net::TcpListener;
use hyper_util::rt::TokioIo;
use mime_guess::from_path;

mod db;

use deadpool_postgres::Pool;

#[derive(Clone)]
struct AppState {
    db_pool: Pool,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    let db_pool = db::create_pool().await;
    let state = Arc::new(AppState { db_pool });

    println!("🚀 Server running on http://localhost:3000");

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream); 

        let state = state.clone();
        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let state = state.clone();
                router(req, state)
            });

            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                eprintln!("Error serving connection: {err}");
            }
        });
    }
}

async fn router(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match req.uri().path() {
        "/" => serve_file("../frontend/index.html", "text/html"),
        path if path.starts_with("/static/") => {
            let file = path.trim_start_matches("/");
            serve_static_file(file)
        }
        path if path.starts_with("/api/") => handle_api(req, state).await,
        _ => Ok(not_found()),
    }
}

fn serve_file(path: &str, content_type: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    match fs::read(path) {
        Ok (contents) => {
            let mut response = Response::new(Full::from(contents));
            let hv = HeaderValue::try_from(content_type).unwrap_or(HeaderValue::from_static("application/octet-stream"));
            response.headers_mut().insert(CONTENT_TYPE, hv);
            Ok(response)
        }
        Err(_) => Ok(not_found()),
    }
}

fn serve_static_file(file_path: &str) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = PathBuf::from("../frontend/").join(file_path);
    match fs::read(&path) {
        Ok(contents) => {
            let mime = from_path(&path).first_or_octet_stream().as_ref().to_string();
            let mut response = Response::new(Full::from(contents));
            response.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_str(&mime).unwrap_or(HeaderValue::from_static("application/octet-stream")),
                );
            Ok(response)
        }
        Err(_) => Ok(not_found()),
    }
}

async fn handle_api(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match req.uri().path() {
        "/api/hello" => {
            let client = state.db_pool.get().await.unwrap();
            let rows = client
                .query("SELECT 'Bonjour depuis Postgres !' AS greeting", &[])
                .await
                .unwrap();
            let greeting: &str = rows[0].get("greeting");

            let html = format!("<div>{greeting}</div>");
            let mut response = Response::new(Full::from(Bytes::from(html)));
            response.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_static("text/html"),
            );
            Ok(response)
        },
        "/api/register" => {
            let collected = req.into_body().collect().await.unwrap();
            let whole_body: Bytes = collected.to_bytes();
            let body_str = String::from_utf8_lossy(&whole_body);
            let params: Vec<(&str, &str)> = body_str
                .split('&')
                .filter_map(|kv| {
                    let mut split = kv.splitn(2, '=');
                    Some((split.next()?, split.next()?))
                })
                .collect();

            let mut username = "";
            let mut password = "";

            for (k, v) in params {
                if k == "username" {
                    username = v;
                } else if k == "password" {
                    password = v;
                }
            }
            // decode URL encoding
            let username = urlencoding::decode(username).unwrap_or_default().to_string();
            let password = urlencoding::decode(password).unwrap_or_default().to_string();

            let pool = &state.db_pool;

            match db::create_user(pool, &username, &password).await {
                Ok(_) => {
                    let mut response = Response::new(Full::from(Bytes::from("✅ Compte créé avec succès !")));
                    response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
                    Ok(response)
                },
                Err(e) => {
                    let msg = format!("❌ Erreur : {e}");
                    let mut response = Response::new(Full::from(Bytes::from(msg)));
                    response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
                    Ok(response)
                }
            }
        },
        _ => Ok(not_found()),
    }
}

fn not_found() -> Response<Full<Bytes>> {
    let mut res = Response::new(Full::from(Bytes::from("404 not found hihi")));
    *res.status_mut() = StatusCode::NOT_FOUND;
    res
}

