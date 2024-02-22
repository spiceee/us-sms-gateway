#![allow(unused_imports)]
use actix_web::{
    http::Method, middleware::Logger, web, App, HttpResponse, HttpServer, Responder, Result,
};
use dotenvy::dotenv;
use redis::aio::ConnectionManager;
use std::env;

mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct IncomingMessage {
        #[serde(rename = "MessageSid")]
        pub message_sid: String,
        #[serde(rename = "SmsSid")]
        pub sms_id: Option<String>,
        #[serde(rename = "SmsMessageSid")]
        pub sms_message_sid: Option<String>,
        #[serde(rename = "AccountSid")]
        pub account_sid: Option<String>,
        #[serde(rename = "MessagingServiceSid")]
        pub messaging_service_sid: Option<String>,
        #[serde(rename = "From")]
        pub from: String,
        #[serde(rename = "To")]
        pub to: String,
        #[serde(rename = "Body")]
        pub body: String,
        #[serde(rename = "NumMedia")]
        pub num_media: Option<String>,
        #[serde(rename = "NumSegments")]
        pub num_segments: Option<String>,
    }
}

mod handlers {
    use crate::models::IncomingMessage;
    use actix_web::middleware::Logger;
    use actix_web::{web, Error, HttpRequest, HttpResponse};
    use redis::aio::ConnectionManager;
    use redis::AsyncCommands;

    pub async fn record_incoming_message(
        params: web::Form<IncomingMessage>,
        redis_conn: web::Data<ConnectionManager>,
        _req: HttpRequest,
    ) -> Result<HttpResponse, Error> {
        Logger::new("New message {params:?}");

        let mut redis = redis_conn.get_ref().clone();
        let _: () = redis
            .set(
                params.message_sid.clone(),
                format!("From: {} Body: {}", params.from, params.body),
            )
            .await
            .expect("Failed to write to Redis");

        Ok(HttpResponse::Ok().content_type("text/xml").body(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Response><Message></Message></Response>"
                .to_string(),
        ))
    }

    pub async fn healthcheck(_req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::Ok()
            .content_type("text/plain")
            .body("Hello!".to_string()))
    }
}

async fn default_handler(_req_method: Method) -> Result<impl Responder> {
    Ok(HttpResponse::MethodNotAllowed().finish())
}

use handlers::{healthcheck, record_incoming_message};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let port = env::var("PORT").expect("Missing port number");
    let port = port.parse::<u16>().expect("Invalid port given");

    let _token = env::var("PRIVATE_EXCHANGE_TOKEN").expect("Missing port number");
    let redis_url = env::var("REDIS_PRIVATE_URL").expect("Missing Redis URL");

    let client = redis::Client::open(redis_url).unwrap();
    let backend = ConnectionManager::new(client).await.unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(backend.clone()))
            .service(web::resource("/incoming").route(web::post().to(record_incoming_message)))
            .service(web::resource("/healthcheck").route(web::get().to(healthcheck)))
            .default_service(web::to(default_handler))
    })
    .bind(("0.0.0.0", port))?
    .run();

    println!("Server running at http://0.0.0.0:{}/", port);

    server.await
}
