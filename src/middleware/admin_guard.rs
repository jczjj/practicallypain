use actix_web::{
    body::{EitherBody, BoxBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::{rc::Rc, task::{Context, Poll}};
use crate::models::user::Claims;

pub struct AdminGuard;

impl<S, B> Transform<S, ServiceRequest> for AdminGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminGuardMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AdminGuardMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {

        let claims_opt = req.extensions().get::<Claims>().cloned();

        if let Some(claims) = claims_opt {
            if claims.is_admin {
                let fut = self.service.call(req);
                return Box::pin(async move {
                    fut.await.map(|res| res.map_into_left_body())
                });
            }
        }

        let (req, _pl) = req.into_parts();
        let response = HttpResponse::Forbidden()
            .json(serde_json::json!({ "error": "Admin access required" }))
            .map_into_right_body(); 

        let res = ServiceResponse::new(req, response);
        Box::pin(async { Ok(res) })
    }
}
