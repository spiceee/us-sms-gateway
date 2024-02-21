#![allow(unused_imports, non_snake_case)] // refinery uses __ in migration filenames
use actix_web::{
    get,
    http::{Method, StatusCode},
    web, App, Either, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config_ = Config::builder()
        .add_source(::config::Environment::default())
        .build()
        .unwrap();

    let config: ExampleConfig = config_.try_deserialize().unwrap();
    let pool = config.pg.create_pool(None, NoTls).unwrap();

    // Perform migrations
    let mut conn = pool.get().await.unwrap();
    let client = conn.deref_mut();

    let token = env::var("PRIVATE_EXCHANGE_TOKEN").expect("Missing port number");
    let redis_url = env::var("REDIS_PRIVATE_URL").expect("Missing Redis URL");

    let client = redis::Client::open(redis_url).unwrap();
    let manager = ConnectionManager::new(client).await.unwrap();
    let backend = RedisBackend::builder(manager).build();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(backend.clone()))
            .service(web::resource("/posts.json").route(web::post().to(add_tracking_code)))
            .default_service(web::to(default_handler))
    })
    .bind(("0.0.0.0", port))?
    .run();

    println!("Server running at http://{}/", config.server_addr);

    server.await
}
