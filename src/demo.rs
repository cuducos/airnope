use actix_cors::Cors;
use actix_web::{
    web::{self, Data, Json},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use airnope::{embeddings::Embeddings, is_spam};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{sync::Mutex, time::Duration};

const LIMIT: Duration = Duration::from_secs(5);
const DEFAULT_IP: &str = "0.0.0.0";

#[derive(Deserialize)]
struct Payload {
    message: String,
}

#[derive(Serialize)]
struct Response {
    spam: bool,
}

async fn handle_request(
    request: HttpRequest,
    payload: Result<Json<Payload>, Error>,
    cache: Data<Cache<String, bool>>,
    embeddings: Data<Arc<Mutex<Embeddings>>>,
) -> actix_web::Result<HttpResponse, Error> {
    let ip = request
        .connection_info()
        .realip_remote_addr()
        .unwrap_or(DEFAULT_IP)
        .to_string();
    if cache.get(&ip).await.is_some() {
        return Ok(HttpResponse::TooManyRequests().body(""));
    }
    cache.insert(ip, true).await;
    match payload {
        Ok(data) => match is_spam(&embeddings, &data.message).await {
            Ok(spam) => Ok(HttpResponse::Ok().json(Response { spam })),
            Err(e) => {
                log::error!("{:?}", e);
                Ok(HttpResponse::InternalServerError().body(""))
            }
        },
        Err(e) => {
            log::error!("{:?}", e);
            Ok(HttpResponse::BadRequest().body(""))
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "24601".to_string())
        .parse()?;
    let cache: Cache<String, bool> = Cache::builder().time_to_live(LIMIT).build();
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?));
    log::info!(
        "Starting AirNope web API on https://{}:{}",
        DEFAULT_IP,
        port
    );
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(Data::new(cache.clone()))
            .app_data(Data::new(Arc::clone(&embeddings)))
            .wrap(
                Cors::default()
                    .allow_any_method()
                    .allow_any_origin()
                    .allow_any_header()
                    .supports_credentials(),
            )
            .service(web::resource("/").route(web::post().to(handle_request)))
    })
    .bind(SocketAddr::from(([0, 0, 0, 0], port)))?
    .run()
    .await?)
}
