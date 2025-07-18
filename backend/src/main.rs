use std::{convert::Infallible, fs, path::PathBuf, sync::Arc };
use hyper::{
    body::{Bytes, Incoming}, header::{HeaderValue, CONTENT_TYPE}, server::conn::http1, service::service_fn, Request, Response, StatusCode, Method
};
use http_body_util::{Full, BodyExt};
use tokio::net::TcpListener;
use hyper_util::rt::TokioIo;
use url::Url;

mod db;
mod models;

use deadpool_postgres::Pool;
use crate::models::{NewRestaurant, ApiRequest};

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
        "/dashboard" => {
            let maybe_token = req.headers()
                .get("cookie")
                .and_then(|v| v.to_str().ok())
                .and_then(|cookie| {
                    cookie.split(';')
                        .find(|s| s.trim().starts_with("session="))
                        .map(|s| s.trim().trim_start_matches("session=").to_string())
                });

            if let Some(token) = maybe_token {
                match db::get_user_from_session(&state.db_pool, &token).await {
                    Ok(Some(_user_id)) => {
                        serve_file("../frontend/dashboard.html", "text/html")
                    },
                    _ => Ok(redirect_to_login()),
                }
            } else {
                Ok(redirect_to_login())
            }
        }
        path if path.ends_with(".html") => {
            let path = format!("../frontend{path}");
            serve_file(&path, "text/html")
        },
        _ => Ok(not_found()),
    }
}

fn redirect_to_login() -> Response<Full<Bytes>> {
    let html = r#"<script>window.location.href='/login.html';</script>"#;
    let mut res = Response::new(Full::from(Bytes::from(html)));
    res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
    res
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
            let header_value = match path.extension().and_then(std::ffi::OsStr::to_str) {
                Some("js") => HeaderValue::from_static("application/javascript"),
                Some("css") => HeaderValue::from_static("text/css"),
                Some("html") => HeaderValue::from_static("text/html"),
                Some("ico") => HeaderValue::from_static("image/x-icon"),
                _ => {
                    let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
                    HeaderValue::from_str(mime_type.as_ref()).unwrap()
                }
            };

            let mut response = Response::new(Full::from(contents));
            response.headers_mut().insert(CONTENT_TYPE, header_value);
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
        "/api/userinfo" => {
            let maybe_token = req.headers()
                .get("cookie")
                .and_then(|v| v.to_str().ok())
                .and_then(|cookie| {
                    cookie.split(';')
                        .find(|s| s.trim().starts_with("session="))
                        .map(|s| s.trim().trim_start_matches("session=").to_string())
                });

            if let Some(token) = maybe_token {
                if let Ok(Some(user_id)) = db::get_user_from_session(&state.db_pool, &token).await {
                    let client = state.db_pool.get().await.unwrap();
                    let row = client
                        .query_one("SELECT username FROM users WHERE id = $1", &[&user_id])
                        .await
                        .unwrap();
                    let username: String = row.get("username");

                    let html = format!("👋 Bonjour, <strong>{username}</strong> !");
                    let mut res = Response::new(Full::from(Bytes::from(html)));
                    res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
                    return Ok(res);
                }
            }

            let mut res = Response::new(Full::from(Bytes::from("Utilisateur inconnu.")));
            res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
            Ok(res)
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
        "/api/login" => {
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

            let username = urlencoding::decode(username).unwrap_or_default().to_string();
            let password = urlencoding::decode(password).unwrap_or_default().to_string();

            let pool = &state.db_pool;

            match db::verify_user(pool, &username, &password).await {
                Ok(true) => {
                    let token = db::create_session(pool, &username).await.unwrap();

                    let mut response = Response::new(Full::from(Bytes::from(r#"<script>window.location.href='/dashboard';</script>"#)));
                    response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));

                    let cookie_value = format!("session={token}; Path=/; HttpOnly");
                    response.headers_mut().insert("Set-Cookie", HeaderValue::from_str(&cookie_value).unwrap());

                    Ok(response)
                },
                Ok(false) => {
                    let mut response = Response::new(Full::from(Bytes::from("❌ Identifiants incorrects")));
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
        "/api/logout" => {
            let mut response = Response::new(Full::from(Bytes::from(
                        r#"<script>window.location.href = "/login.html";</script>"#
            )));
            response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));

            let expired_cookie = "session=deleted; Path=/; Max-Age=0; HttpOnly";
            response.headers_mut().insert("Set-Cookie", HeaderValue::from_static(expired_cookie));

            Ok(response)
        },
        "/api/restaurants" => {
            let state_clone_for_auth = Arc::clone(&state); 

            let maybe_user_id_future = if let Some(cookie_str) = req.headers().get("cookie").and_then(|v| v.to_str().ok()) {
                cookie_str.split(';')
                    .find(|s| s.trim().starts_with("session="))
                    .map(|s| s.trim().trim_start_matches("session=").to_string())
                    .map(|token| async move {
                        db::get_user_from_session(&state_clone_for_auth.db_pool, &token).await.ok().flatten()
                    })
            } else {
                None
            };

            let user_id = if let Some(fut) = maybe_user_id_future {
                fut.await
            } else {
                None
            };

            if user_id.is_none() {
                let mut res = Response::new(Full::from(Bytes::from("{\"error\":\"Authentification requise\"}")));
                *res.status_mut() = StatusCode::UNAUTHORIZED;
                res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                return Ok(res);
            }
            let user_id = user_id.unwrap();

            match *req.method() {
                Method::GET => {
                    let full_url = format!("http://localhost{}", req.uri().path_and_query().unwrap().as_str());
                    let url = Url::parse(&full_url).unwrap();

                    let mut favorites_only = false;
                    let mut limit: Option<i64> = None;

                    for (key, val) in url.query_pairs() {
                        match key.as_ref() {
                            "favorites" if val == "true" => favorites_only = true,
                            "limit" => limit = val.parse::<i64>().ok(),
                            _ => {} // On ignore les autres paramètres
                        }
                    }

                    match db::get_restaurants_for_user(&state.db_pool, user_id, favorites_only, limit).await {
                        Ok(restaurants) => {
                            // Si la liste est vide, on renvoie un message
                            if restaurants.is_empty() {
                                let message = if favorites_only {
                                    "<p class='text-gray-500'>Vous n'avez encore aucun restaurant favori.</p>"
                                } else {
                                    "<p class='text-gray-500'>Vous n'avez pas encore ajouté de restaurant.</p>"
                                };
                                let mut res = Response::new(Full::from(Bytes::from(message)));
                                res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
                                return Ok(res);
                            }

                            // On construit la liste HTML des restaurants
                            let mut html_list = String::from("<ul class='space-y-4'>");
                            for r in restaurants {
                                let favorite_icon = if r.is_favorite { "❤️" } else { "🤍" };
                                let rating_stars = "⭐".repeat(r.rating.unwrap_or(0) as usize);

                                let item = format!(
                                    r#"<li class='p-4 border rounded-lg shadow-sm bg-white'>
                                            <div class='flex justify-between items-start'>
                                                <div>
                                                    <h4 class='font-bold text-lg text-gray-800'>{name}</h4>
                                                    <p class='text-sm text-gray-500'>{city}</p>
                                                </div>
                                                <span class='text-xl'>{icon}</span>
                                            </div>
                                            <p class='text-gray-700 mt-2'>{description}</p>
                                            <p class='text-yellow-500 font-bold mt-2'>{rating}</p>
                                        </li>"#,
                                        name = r.name,
                                        city = r.city,
                                        icon = favorite_icon,
                                        description = r.description.as_deref().unwrap_or(""),
                                        rating = rating_stars
                                );
                                html_list.push_str(&item);
                            }
                            html_list.push_str("</ul>");

                            let mut res = Response::new(Full::from(Bytes::from(html_list)));
                            res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
                            Ok(res)
                        },
                        Err(_) => {
                            let error_html = "<p class='text-red-500'>Erreur lors de la récupération des restaurants.</p>";
                            let mut res = Response::new(Full::from(Bytes::from(error_html)));
                            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
                            Ok(res)
                        }
                    }
                },
                Method::POST => {
                    let collected = req.into_body().collect().await.unwrap();
                    let body = collected.to_bytes();

                    println!("\n--- CORPS DE LA REQUÊTE REÇU ---\n{}\n--------------------------------\n", String::from_utf8_lossy(&body));
                    match serde_json::from_slice::<ApiRequest>(&body) {
                        Ok(api_request) => {
                            let new_restaurant = api_request.data;

                            match db::create_restaurant(&state.db_pool, user_id, &new_restaurant).await {
                                Ok(restaurant) => {
                                    let json = serde_json::to_string(&restaurant).unwrap();
                                    let mut res = Response::new(Full::from(Bytes::from(json)));
                                    *res.status_mut() = StatusCode::CREATED;
                                    res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                                    res.headers_mut().insert("HX-Trigger", HeaderValue::from_static("restaurant-added"));
                                    Ok(res)
                                },
                                Err(_) => {
                                    let mut res = Response::new(Full::from(Bytes::from("{\"error\":\"Erreur interne du serveur\"}")));
                                    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                    res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                                    Ok(res)
                                }
                            }
                        },
                        Err(e) => {
                            let msg = format!("{{\"error\":\"Données invalides: {e}\"}}");
                            let mut res = Response::new(Full::from(Bytes::from(msg)));
                            *res.status_mut() = StatusCode::BAD_REQUEST;
                            res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                            Ok(res)
                        }
                    }
                },

                _ => {
                    let mut res = Response::new(Full::from(Bytes::from("{\"error\":\"Méthode non autorisée\"}")));
                    *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                    res.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                    Ok(res)
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

