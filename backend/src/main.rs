use std::{convert::Infallible, fs, path::PathBuf};
use hyper::{
    body::{Bytes, Incoming}, header::{HeaderValue, CONTENT_TYPE}, server::conn::http1, service::service_fn, Request, Response, StatusCode
};
use http_body_util::Full;
use tokio::net::TcpListener;
use hyper_util::rt::TokioIo;
use mime_guess::from_path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("🚀 Server running on http://localhost:3000");

    loop {
        let (stream, _) = listener.accept().await?;
        let service = service_fn(router);

        let io = TokioIo::new(stream); 

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                eprintln!("Error serving connection: {err}");
            }
        });
    }
}

async fn router(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match req.uri().path() {
        "/" => serve_file("../frontend/index.html", "text/html"),
        path if path.starts_with("/static/") => {
            let file = path.trim_start_matches("/");
            serve_static_file(file)
        }
        path if path.starts_with("/api/") => handle_api(req).await,
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
    let path = PathBuf::from("frontend").join(file_path);
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

async fn handle_api(_req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let html = "<div> Réponse HTMX arrive d'ici peu t'inquiètes </div>";
    let mut response = Response::new(Full::from(Bytes::from(html)));
    response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("text/html"),
        );
    Ok(response)
}

fn not_found() -> Response<Full<Bytes>> {
    let mut res = Response::new(Full::from(Bytes::from("404 not found hihi")));
    *res.status_mut() = StatusCode::NOT_FOUND;
    res
}

