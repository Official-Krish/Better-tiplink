use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::rc::Rc;

use crate::auth::verify_jwt;

pub struct AuthMiddleware;

impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    // require the inner service to use BoxBody for its response body
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            // Check for Authorization header
            let auth_header = req.headers().get("Authorization");

            if let Some(auth_header) = auth_header {
                if let Ok(token) = auth_header.to_str() {
                    // Remove "Bearer " prefix if present
                    let token = if token.starts_with("Bearer ") {
                        &token[7..]
                    } else {
                        token
                    };

                    match verify_jwt(token) {
                        Ok(_payload) => {
                            // Token is valid, proceed with the request
                            service.call(req).await
                        }
                        Err(_) => {
                            // Token is invalid -> build ServiceResponse<BoxBody>
                            let response = HttpResponse::Unauthorized()
                                .json(serde_json::json!({"error": "Invalid token"}));
                            // Convert ServiceRequest + HttpResponse -> ServiceResponse<BoxBody>
                            let srv_resp = req.into_response(response.map_into_boxed_body());
                            Ok(srv_resp)
                        }
                    }
                } else {
                    // Authorization header is not a valid string
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({"error": "Invalid authorization header"}));
                    let srv_resp = req.into_response(response.map_into_boxed_body());
                    Ok(srv_resp)
                }
            } else {
                // No Authorization header present
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({"error": "Missing authorization header"}));
                let srv_resp = req.into_response(response.map_into_boxed_body());
                Ok(srv_resp)
            }
        })
    }
}
