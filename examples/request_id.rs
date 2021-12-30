use actix_web::{get, middleware::Logger, App, HttpRequest, HttpServer, Responder};
use actix_web_requestid::{RequestID, RequestIdMiddleware};

#[get("/test")]
async fn test(id: RequestID, req: HttpRequest) -> impl Responder {
    format!(
        "RequestID: {}\nHead:{:#?}",
        &id,
        req.headers().get("my_request_id")
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let id_middleware = RequestIdMiddleware::new("my_request_id");
        // You need to wrap the Logger first and then the RequestIdMiddleware. Otherwise, the log will not record "my_request_id".
        App::new()
            .wrap(Logger::new(&id_middleware.log_format()))
            .wrap(id_middleware)
            .service(test)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
