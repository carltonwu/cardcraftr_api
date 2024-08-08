#![allow(unused)]

pub use self::error::{Error, Result};

use axum::extract::{Path, Query};
use axum::http::{Method, Uri};
use axum::{middleware, Json};
use axum::response::{IntoResponse, Response};
use axum::{response::Html};
use axum::routing::{get, get_service, Router};
use ctx::Ctx;
use log::log_request;
use model::ModelController;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;
use tower_cookies::{CookieManager, CookieManagerLayer};
use tower_http::services::ServeDir;
use uuid::Uuid;
use web::mw_auth::mw_require_auth;

mod log;
mod ctx;
mod error;
mod model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {

    // initialize mc
    let mc = ModelController::new().await?;

    let routes_api = web::routes_cards::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_api)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static());

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("->> LISTENING on {:?}\n", listener.local_addr());
    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });
            println!("  ->> client_error_body: {client_error_body}");

            (*status_code, Json(client_error_body)).into_response()
        });

    println!("    ->> server log line - {uuid} - Error: {service_error:?}");

    let client_error = client_status_error.unzip().1;
    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    println!();
    error_response.unwrap_or(res)
}

// region: -- Routes Hello

fn routes_hello() -> Router {
    Router::new()
    .route("/hello",get(handler_hello))
    .route("/hello2/:name", get(handler_hello2))
}

fn routes_static() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World!");
    Html(format!("Hello<strong> {name}</strong>"))
}

async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {name:?}", "HANDLER");

    Html(format!("Hello<strong> {name}</strong>"))
}