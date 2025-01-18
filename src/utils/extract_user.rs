use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;

use crate::utils::jwt;
//Il vérifie que l'utilisateur est authentifié grâce au token JWT.
pub struct ExtractUser;

impl<S, B> Transform<S, ServiceRequest> for ExtractUser
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ExtractUserMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ExtractUserMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct ExtractUserMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ExtractUserMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx) // Appel explicite à `poll_ready`
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
    
        Box::pin(async move {
            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        match jwt::validate_token(token) {
                            Ok(claims) => {
                                match claims.sub.parse::<i32>() {
                                    Ok(user_id) => {
                                        req.extensions_mut().insert(user_id);
                                    }
                                    Err(_) => {
                                        return Err(actix_web::error::ErrorUnauthorized("Identifiant utilisateur invalide dans le JWT"));
                                    }
                                }
                            }
                            Err(_) => {
                                return Err(actix_web::error::ErrorUnauthorized("Jeton JWT invalide ou expiré"));
                            }
                        }
                    }
                }
            }
    
            service.call(req).await
        })
    }
    
}
