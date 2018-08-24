# Actix-web-requestid 

[![Build Status](https://travis-ci.com/pastjean/actix-web-requestid.svg?branch=master)](https://travis-ci.com/pastjean/actix-web-requestid)
[![codecov](https://codecov.io/gh/pastjean/actix-web-requestid/branch/master/graph/badge.svg)](https://codecov.io/gh/pastjean/actix-web-requestid) 
[![crates.io](https://meritbadge.herokuapp.com/actix-web-requestid)](https://crates.io/crates/actix-web-requestid)
[![License](https://img.shields.io/crates/l/actix-web-requestid.svg)](https://github.com/pastjean/actix-web-requestid#license)

A rust library to add a requestid with the actix-web framework.

# Usage

Add this to your Cargo.toml:

```toml
[dependencies]
actix-web-requestid = "0.1.0"
```

And this to your crate root:

```rust
extern crate actix_web;
extern crate actix_web_requesid;

use actix_web::{http, server, App, Path, Responder};
use actix_web_requestid::RequestIDHeader

fn index(info: Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", info.1, info.0)
}

fn main() {
    server::new(
        || App::new()
            .middleware(RequestIDHeader)
            .route("/{id}/{name}/index.html", http::Method::GET, index))
        .bind("127.0.0.1:8080").unwrap()
        .run();
}
```

# License

actix-web-requestid is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).