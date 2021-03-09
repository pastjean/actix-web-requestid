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
actix-web-requestid = "0.4.0"
```

And this to your crate root:

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_web_requestid::{RequestID, RequestIDService};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(
        || App::new()
            .wrap(RequestIDService::new())
            .service(web::resource("/").to(|| HttpResponse::Ok())))
        .bind("127.0.0.1:59880")?
        .run()
        .await
}
```

# License

actix-web-requestid is distributed under the terms of both the MIT license and the Apache License (Version 2.0).
