use actix_web::{
    body::{EitherBody, BoxBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::{rc::Rc, task::{Context, Poll}};
use crate::handlers::auth::validate_jwt;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
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
        println!("AuthMiddleware: processing request to {}", req.path());

        let auth_header = req.headers().get("Authorization").and_then(|hv| hv.to_str().ok());

        if req.path() == "/login" {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                println!("⬅ Outgoing: {} {}", res.status(), res.request().uri()); // Log outgoing response
                Ok(res.map_into_left_body())
            });
        }
         let token_opt = req.cookie("auth_token").map(|c| c.value().to_string());
         
        if let Some(token) = token_opt {
                match validate_jwt(&token) {
                    Ok(data) => {
                        println!("AuthMiddleware: JWT valid for user {}", data.claims.sub);
                        req.extensions_mut().insert(data.claims);
                        let fut = self.service.call(req);
                        return Box::pin(async move {
                            let res = fut.await?;
                            println!("AuthMiddleware: downstream response status: {}", res.status());
                            Ok(res.map_into_left_body())
                        });
                    }
                    Err(err) => {
                        println!("AuthMiddleware: JWT validation failed: {:?}", err);
                    }
                }
            } else {
                println!("AuthMiddleware: No auth_token cookie found");
            }

            // Unauthorized if token is missing or invalid
            let (req, _pl) = req.into_parts();
            let response = HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "Unauthorized" }))
                .map_into_right_body();

            let res = ServiceResponse::new(req, response);
            Box::pin(async { Ok(res) })
        }
}
