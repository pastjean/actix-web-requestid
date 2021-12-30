//! actix-web-requestid.
//!
//! [`RequestID`] provides a "request-id" to a http request. This can be
//! used for tracing, debuging, user error reporting.
//!
//! Insert the request id middleware to provide the request-id to the
//! `request-id` http header. To access requestID data, [`RequestID`] actix
//!  extractor must be used.
//!
//! It is still useable without the middleware. The first time you try to
//! extract the id, it will be generated. Then reused along the request.
//! You can for exemple use that in a Logging or tracing middleware.
use actix_web::{
    dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderName, HeaderValue},
    Error, FromRequest, HttpMessage, HttpRequest,
};
use futures_util::future::{ok, ready, FutureExt, LocalBoxFuture, Ready};
use log::warn;
use std::convert::Infallible;
use std::ops::Deref;
use std::task::{Context, Poll};

pub const DEFAULT_ID_HEAD_NAME: &'static str = "request_id";

#[derive(Debug, Clone)]
pub struct RequestIdMiddleware {
    pub head_name: &'static str,
}

impl Default for RequestIdMiddleware {
    fn default() -> Self {
        RequestIdMiddleware {
            head_name: DEFAULT_ID_HEAD_NAME,
        }
    }
}

impl RequestIdMiddleware {
    pub fn new(head_name: &'static str) -> Self {
        Self { head_name }
    }
    pub fn log_format(&self) -> String {
        format!(
            "[%{{{}}}i] %a %r %s %b %{{Referer}}i %{{User-Agent}}i %T",
            self.head_name
        )
    }
}

impl<S, B> Transform<S> for RequestIdMiddleware
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestIdService {
            service,
            head_name: self.head_name,
        })
    }
}

pub struct RequestIdService<S> {
    service: S,
    head_name: &'static str,
}

impl<S, B> Service for RequestIdService<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    #[allow(clippy::borrow_interior_mutable_const)]
    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let id_head = self.head_name;

        let req_id = req.headers().get(id_head).map(|hv|{
           match hv.to_str() {
                Ok(raw_id) => RequestID {inner: raw_id.to_string()},
                Err(err) => {
                    let new_id=  RequestID::new();
                    warn!(
                        "This request header allows only visible ASCII characters, which will be overwritten. error:{}, id:{}, head:{}",
                        err,&new_id, id_head
                    );
                    new_id
                }
            }
        }).unwrap_or(RequestID::new());

        req.headers_mut().insert(
            HeaderName::from_static(self.head_name),
            HeaderValue::from_str(&req_id).unwrap(),
        );
        req.extensions_mut().insert(req_id.clone());

        let fut = self.service.call(req);

        async move {
            let mut res = fut.await?;

            if !res.headers().contains_key(id_head) {
                res.headers_mut().insert(
                    HeaderName::from_static(id_head),
                    HeaderValue::from_str(&req_id).unwrap(),
                );
            }
            Ok(res)
        }
        .boxed_local()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestID {
    inner: String,
}

impl RequestID {
    pub fn new() -> Self {
        Self {
            inner: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl std::fmt::Display for RequestID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl Deref for RequestID {
    type Target = String;
    fn deref(&self) -> &String {
        &self.inner
    }
}

impl FromRequest for RequestID {
    type Error = Infallible;
    type Future = Ready<Result<RequestID, Infallible>>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let id = match req.extensions().get::<RequestID>() {
            Some(id) => id.clone(),
            None => RequestID::new(),
        };
        ready(Ok(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, web, App, HttpResponse};

    #[actix_rt::test]
    async fn test_none_head() {
        let head_name = "my_id";

        let mut app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware::new(head_name))
                .service(web::resource("/").to(
                    move |id: RequestID, req: HttpRequest| async move {
                        assert!(!id.is_empty());
                        assert_eq!(req.headers().get(head_name).unwrap().to_str().unwrap(), *id);
                        HttpResponse::Ok().await
                    },
                )),
        )
        .await;

        let req = test::TestRequest::with_uri("/").to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        assert!(!resp.headers().get(head_name).unwrap().is_empty());
    }

    #[actix_rt::test]
    async fn test_exist_head() {
        let value = "123456";
        let mut app = test::init_service(App::new().wrap(RequestIdMiddleware::default()).service(
            web::resource("/").to(move |id: RequestID, req: HttpRequest| async move {
                assert_eq!(*id, value);
                assert_eq!(req.headers().get(DEFAULT_ID_HEAD_NAME).unwrap(), value);
                HttpResponse::Ok().await
            }),
        ))
        .await;

        let req = test::TestRequest::with_uri("/")
            .header(DEFAULT_ID_HEAD_NAME, value)
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        assert_eq!(resp.headers().get(DEFAULT_ID_HEAD_NAME).unwrap(), value);
    }
}
