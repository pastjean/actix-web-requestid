# Actix-web-requestid 

[![CI](https://github.com/pastjean/actix-web-requestid/workflows/CI/badge.svg)](https://github.com/pastjean/actix-web-requestid/actions?query=workflow%3ACI)
[![crates.io](https://meritbadge.herokuapp.com/actix-web-requestid)](https://crates.io/crates/actix-web-requestid)
[![Documentation](https://docs.rs/actix-web-requestid/badge.svg)](https://docs.rs/actix-web-requestid)
[![License](https://img.shields.io/crates/l/actix-web-requestid.svg)](https://github.com/pastjean/actix-web-requestid#license)

A rust library to add a requestid with the actix-web framework.

# Usage

Add this to your Cargo.toml:

```toml
[dependencies]
actix-web-requestid = "2.0.0-beta.1"
```

And this to your crate root:

```rust
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
```

# License

actix-web-requestid is distributed under the terms of both the MIT license and the Apache License (Version 2.0).
