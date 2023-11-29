use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error };
use std::{
    rc::Rc,
    future::{ready, Ready}
};
use crate::middlewares::jwt::JwtMiddleware;

pub struct AuthRequired;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S: 'static, B> Transform<S, ServiceRequest> for AuthRequired
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddleware { service: Rc::new(service) }))
    }
}

