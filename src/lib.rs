//! Request-id.
//!
//! RequestID provides a "request-id" to a http request. This can be
//! used for tracing, debuging, user error reporting.
//!
//! In general, you insert a *request-id* middleware and initialize it
//! To access session data, [*RequestID*](struct.RequestID.html) extractor
//!  must be used.

extern crate actix_web;
extern crate futures;
extern crate rand;

use actix_web::dev::{Payload, Service, Transform, ServiceRequest, ServiceResponse};
use actix_web::http::{HeaderName, HeaderValue};
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest};
use futures::future::{ok, Ready};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// The header set by the middleware
pub const REQUEST_ID_HEADER: &str = "request-id";


pub trait RequestIDMessage {
    fn id(&self) -> String;
}

impl RequestIDMessage for RequestID {
    fn id(&self) -> String {
        self.0.id()
    }
}


#[derive(Clone)]
pub struct RequestID(HttpRequest);

struct RequestIDItem(String);

impl<T> RequestIDMessage for T
where
    T: HttpMessage,
{
    fn id(&self) -> String {
        if let Some(id) = self.extensions().get::<RequestIDItem>() {
            return id.0.clone();
        }

        let id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>();
        self.extensions_mut().insert(RequestIDItem(id.clone()));

        return id;
    }
}

impl FromRequest for RequestID {
    type Error = Error;
    type Future = Ready<Result<RequestID, Error>>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ok(RequestID(req.clone()))
    }
}

pub struct RequestIDService;

impl RequestIDService {
    /// Create new identity service with specified backend.
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, B> Transform<S> for RequestIDService
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIDServiceMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestIDServiceMiddleware { service: service })
    }
}

#[doc(hidden)]
pub struct RequestIDServiceMiddleware<S> {
    service: S,
}

impl<S, B> Service for RequestIDServiceMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let req_id = req.id(); //RequestID(req).id();
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let name = HeaderName::from_static(REQUEST_ID_HEADER);
            let val = HeaderValue::from_str(&req_id).unwrap();
            res.headers_mut().insert(name, val);

            println!("{:?}",res.headers());
            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;
    use actix_web::{test, web, App, HttpResponse, http::StatusCode};

    #[test]
    fn request_id_is_consistent_for_same_request() {
        let req = TestRequest::default().to_http_request();

        let req_id = RequestID(req);

        assert_eq!(req_id.id(), req_id.id());
    }

    #[test]
    fn request_id_is_new_between_different_requests() {
        let req1 = TestRequest::default().to_http_request();
        let req2 = TestRequest::default().to_http_request();

        let req_id1 = RequestID(req1);
        let req_id2 = RequestID(req2);

        assert_eq!(req_id1.id(), req_id1.id());
        assert_eq!(req_id2.id(), req_id2.id());
        assert!(req_id1.id() != req_id2.id());
    }

    #[actix_rt::test]
    async fn middleware_adds_request_id_in_headers() {
        let mut app = test::init_service(
            App::new()
                .wrap(RequestIDService::new())
                .service(web::resource("/").to(|| async { HttpResponse::Ok() }))
        ).await;
    
        // Create request object
        let req = test::TestRequest::with_uri("/").to_request();
    
        // Execute application
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        //println!("{:?}",resp.headers().get("request-id").map(|h| h.to_str()));
        assert!(!resp.headers().get("request-id").unwrap().is_empty());
    }
}
