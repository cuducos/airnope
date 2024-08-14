use actix_cors::Cors;
use actix_web::{
    web::{self, Data, Json},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use airnope::{embeddings::Embeddings, is_spam};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{
    spawn,
    sync::Mutex,
    time::{sleep, Duration, Instant},
};

const LIMIT: Duration = Duration::from_secs(5);
const CLEAN_UP_EVERY: Duration = Duration::from_secs(60);
const DEFAULT_IP: &str = "0.0.0.0";

#[derive(Deserialize)]
struct Payload {
    message: String,
}

#[derive(Serialize)]
struct Response {
    spam: bool,
}

async fn clean_up_ips(ips: Arc<DashMap<String, Instant>>) {
    loop {
        sleep(CLEAN_UP_EVERY).await;
        let now = Instant::now();
        ips.retain(|_, t| now.duration_since(*t) < LIMIT);
    }
}

async fn handle_request(
    request: HttpRequest,
    payload: Result<Json<Payload>, Error>,
    ips: Data<Arc<DashMap<String, Instant>>>,
    embeddings: Data<Arc<Mutex<Embeddings>>>,
) -> actix_web::Result<HttpResponse, Error> {
    let ip = request
        .connection_info()
        .realip_remote_addr()
        .unwrap_or(DEFAULT_IP)
        .to_string();
    if let Some(last) = ips.get(&ip) {
        let now = Instant::now();
        if now.duration_since(*last) < LIMIT {
            return Ok(HttpResponse::TooManyRequests().body(""));
        }
    }
    ips.insert(ip, Instant::now());
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

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init(); // based on RUST_LOG environment variable
    let ips: Arc<DashMap<String, Instant>> = Arc::new(DashMap::new());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "24601".to_string())
        .parse()?;
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?));
    spawn(clean_up_ips(Arc::clone(&ips)));
    log::info!(
        "Starting AirNope web API on https://{}:{}",
        DEFAULT_IP,
        port
    );
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Arc::clone(&embeddings)))
            .app_data(Data::new(Arc::clone(&ips)))
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
