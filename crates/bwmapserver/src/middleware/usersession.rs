use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::body::EitherBody;
use actix_web::{
    cookie::Cookie,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error, HttpMessage, HttpResponse,
};
use bb8_postgres::{bb8::Pool, tokio_postgres::NoTls, PostgresConnectionManager};
use futures_util::{future::LocalBoxFuture, FutureExt};
use log::info;
use tracing::{instrument, Instrument};

#[derive(Clone, Debug)]
pub struct UserSession {
    pub id: i64,
    pub username: String,
    pub token: String,
}

pub struct UserSessionTransformer;

impl<S: 'static, B> Transform<S, ServiceRequest> for UserSessionTransformer
where
    S: Service<ServiceRequest, Response = ServiceResponse<EitherBody<B>>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = UserSessionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UserSessionMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct UserSessionMiddleware<S> {
    service: Rc<S>,
}

impl<S: 'static, B> Service<ServiceRequest> for UserSessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<EitherBody<B>>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    #[instrument(skip_all, name = "")]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let pool = req
            .app_data::<Data<Pool<PostgresConnectionManager<NoTls>>>>()
            .unwrap()
            .clone();

        let s = self.service.clone();

        async move {
            if let Some(cookie_username) = req.cookie("username") {
                if let Some(cookie_token) = req.cookie("token") {

                    let con = pool.get().await.unwrap();
                    let row = con
                        .query_one(
                            "select id, token, username from account where username = $1",
                            &[&cookie_username.value()],
                        )
                        .await
                        .unwrap();

                    let db_idtoken = (
                        row.get::<_, i64>(0),
                        row.get::<_, String>(1),
                        row.get::<_, String>(2),
                    );

                    if cookie_token.value() == db_idtoken.1.as_str() {
                        info!("id: {}, username: {}, token: {}", db_idtoken.0, db_idtoken.2, db_idtoken.1);
                        req.extensions_mut().insert(UserSession {
                            id: db_idtoken.0,
                            username: db_idtoken.2,
                            token: db_idtoken.1,
                        });

                        Ok(s.call(req).await?)
                    } else {
                        Ok(req.into_response(
                            HttpResponse::MovedPermanently()
                            .insert_header(("location", "/"))
                            .cookie(
                                Cookie::build("username", "")
                                    .path("/")
                                    .same_site(actix_web::cookie::SameSite::Lax)
                                    .secure(true)
                                    .expires(
                                        actix_web::cookie::time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                                    )
                                    .finish(),
                            )
                            .cookie(
                                Cookie::build("token", "")
                                    .path("/")
                                    .same_site(actix_web::cookie::SameSite::Lax)
                                    .secure(true)
                                    .expires(
                                        actix_web::cookie::time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                                    )
                                    .finish(),
                            )
                            .finish()
                                .map_into_right_body(),
                        ))
                    }
                } else {
                    Ok(req.into_response(
                        HttpResponse::MovedPermanently()
                            .insert_header(("location", "/"))
                            .cookie(
                                Cookie::build("username", "")
                                    .path("/")
                                    .same_site(actix_web::cookie::SameSite::Lax)
                                    .secure(true)
                                    .expires(
                                        actix_web::cookie::time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                                    )
                                    .finish(),
                            )
                            .cookie(
                                Cookie::build("token", "")
                                    .path("/")
                                    .same_site(actix_web::cookie::SameSite::Lax)
                                    .secure(true)
                                    .expires(
                                        actix_web::cookie::time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                                    )
                                    .finish(),
                            )
                            .finish()
                            .map_into_right_body(),
                    ))
                }
            } else {
                Ok(s.call(req).await?)
            }
        }
        .instrument(tracing::span::Span::current())
        .boxed_local()
    }
}
