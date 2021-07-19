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
use std::convert::Infallible;
use std::future::{ready, Future, Ready};
use std::pin::Pin;

use actix_web::dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::{HeaderName, HeaderValue};
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest};
use rand::distributions::Alphanumeric;
use rand::Rng;

pub const REQUEST_ID_HEADER: &str = "request-id";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestID {
    inner: String,
}

impl From<RequestID> for String {
    fn from(r: RequestID) -> Self {
        r.inner
    }
}

impl std::fmt::Display for RequestID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

/// Extractor implementation for [`RequestID`] type.
///
/// ```noexec
/// # use actix_web::*;
/// # use actix_web_requestid::{RequestID};
///
/// async fn index(id: RequestID) -> String {
///     format!("Welcome! {}", id)
/// }
/// ```
impl FromRequest for RequestID {
    type Error = Infallible;
    type Future = Ready<Result<RequestID, Infallible>>;
    type Config = ();

    #[inline]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(req.request_id()))
    }
}

/// Request id middleware
///
/// ```
/// use actix_web::*;
/// use actix_web_requestid::{RequestIDMiddleware};
///
/// let app = App::new()
///     .wrap(RequestIDMiddleware::default());
/// ```
pub struct RequestIDMiddleware {}

impl Default for RequestIDMiddleware {
    fn default() -> Self {
        Self {}
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestIDMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIDService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIDService {
            wrapped_service: service,
        }))
    }
}

pub struct RequestIDService<S> {
    wrapped_service: S,
}

impl<S, Req> Service<ServiceRequest> for RequestIDService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<Req>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<Req>;
    type Error = S::Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.wrapped_service.poll_ready(ctx)
    }

    fn call(&self, req: actix_web::dev::ServiceRequest) -> Self::Future {
        let id = req.request_id().inner;
        let fut = self.wrapped_service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            res.headers_mut().append(
                HeaderName::from_static(REQUEST_ID_HEADER),
                HeaderValue::from_str(&id).unwrap(),
            );

            Ok(res)
        })
    }
}

pub trait RequestIDMessage {
    fn request_id(&self) -> RequestID;
}

impl<T> RequestIDMessage for T
where
    T: HttpMessage,
{
    fn request_id(&self) -> RequestID {
        if let Some(id) = self.extensions().get::<RequestID>() {
            return id.clone();
        }

        let new_id = RequestID {
            inner: rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .map(char::from)
                .take(10)
                .collect::<_>(),
        };

        self.extensions_mut().insert(new_id.clone());

        new_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;
    use actix_web::{http::StatusCode, test, web, App, HttpResponse};

    #[actix_rt::test]
    async fn request_id_is_consistent_for_same_request() {
        let req = TestRequest::default().to_http_request();
        let id_1 = RequestID::extract(&req).await.unwrap();
        let id_2 = RequestID::extract(&req).await.unwrap();

        assert_eq!(id_1, id_2);
    }

    #[actix_rt::test]
    async fn request_id_is_new_between_different_requests() {
        let req1 = TestRequest::default().to_http_request();
        let req2 = TestRequest::default().to_http_request();

        let req1_id = RequestID::extract(&req1).await.unwrap();
        let req2_id = RequestID::extract(&req2).await.unwrap();

        assert!(req1_id != req2_id);
    }

    #[actix_rt::test]
    async fn middleware_adds_request_id_in_headers() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIDMiddleware::default())
                .service(web::resource("/").to(|| async { HttpResponse::Ok().await })),
        )
        .await;

        // Create request object
        let req = test::TestRequest::with_uri("/").to_request();

        // Execute application
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        assert!(!resp.headers().get("request-id").unwrap().is_empty());
    }
}
