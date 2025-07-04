use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error};
use futures_util::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::rc::Rc;
use std::time::Instant;

// Entry point for the logging middleware
// This middleware logs the request path and the time taken to process the request.
pub struct Logging;


// Tells Actix how to apply the middleware
impl<S, B> Transform<S, ServiceRequest> for Logging
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    // Below function is called when app starts, builds middleware
    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggingMiddleware {
            service: Rc::new(service),
        })
    }
}

// Actual middlware that wraps every request, References the original service
pub struct LoggingMiddleware<S> {
    service: Rc<S>,
}

// Defines how the middleware will behave for each request
impl<S, B> Service<ServiceRequest> for LoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_owned();
        let svc = self.service.clone();
        let start = Instant::now();

        Box::pin(async move {
            let res = svc.call(req).await;
            let duration = start.elapsed();
            println!("\nRequest: '{}' took: {:?}", path, duration);
            res
        })
    }
}
