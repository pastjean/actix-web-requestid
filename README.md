# Actix-web-requestid 

[![Build Status](https://travis-ci.com/pastjean/actix-web-requestid.svg?branch=master)](https://travis-ci.com/pastjean/actix-web-requestid)
[![codecov](https://codecov.io/gh/pastjean/actix-web-requestid/branch/master/graph/badge.svg)](https://codecov.io/gh/pastjean/actix-web-requestid) 
[![crates.io](https://meritbadge.herokuapp.com/actix-web-requestid)](https://crates.io/crates/actix-web-requestid)
[![Documentation](https://docs.rs/actix-web-requestid/badge.svg)](https://docs.rs/actix-web-requestid)
[![License](https://img.shields.io/crates/l/actix-web-requestid.svg)](https://github.com/pastjean/actix-web-requestid#license)

A rust library to add a requestid with the actix-web framework.

# Usage

Add this to your Cargo.toml:

```toml
[dependencies]
actix-web-requestid = "0.1.2"
```

And this to your crate root:

```rust
extern crate actix_web;
extern crate actix_web_requestid;

use actix_web::{http, server, App, Path, Responder};
use actix_web_requestid::RequestIDHeader

fn main() {
    server::new(
        || App::new()
            .middleware(RequestIDHeader)
            .resource("/", |r| {
                r.method(http::Method::GET).f(|_req| "Hello!");
            }))
        .bind("127.0.0.1:8080").unwrap()
        .run();
}
```

# License

actix-web-requestid is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).