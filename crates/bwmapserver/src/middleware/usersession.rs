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
            let log_out_user = || {
                HttpResponse::MovedPermanently()
                    .insert_header(("location", "/"))
                    .cookie(
                        Cookie::build("username", "")
                            .path("/")
                            .same_site(actix_web::cookie::SameSite::Lax)
                            .secure(true)
                            .expires(
                                actix_web::cookie::time::OffsetDateTime::from_unix_timestamp(0)
                                    .unwrap(),
                            )
                            .finish(),
                    )
                    .cookie(
                        Cookie::build("token", "")
                            .path("/")
                            .same_site(actix_web::cookie::SameSite::Lax)
                            .secure(true)
                            .expires(
                                actix_web::cookie::time::OffsetDateTime::from_unix_timestamp(0)
                                    .unwrap(),
                            )
                            .finish(),
                    )
                    .finish()
                    .map_into_right_body()
            };

            if let Some(cookie_username) = req.cookie("username") {
                if let Some(cookie_token) = req.cookie("token") {
                    let con = pool.get().await.unwrap();
                    let row = con
                        .query_opt(
                            "select id, token, username from account where username = $1",
                            &[&cookie_username.value()],
                        )
                        .await
                        .unwrap();

                    let Some(row) = row else {
                        return Ok(req.into_response(log_out_user()));
                    };

                    let db_idtoken = (
                        row.get::<_, i64>(0),
                        row.get::<_, String>(1),
                        row.get::<_, String>(2),
                    );

                    if cookie_token.value() == db_idtoken.1.as_str() {
                        info!(
                            "id: {}, username: {}, token: {}",
                            db_idtoken.0, db_idtoken.2, db_idtoken.1
                        );
                        req.extensions_mut().insert(UserSession {
                            id: db_idtoken.0,
                            username: db_idtoken.2,
                            token: db_idtoken.1,
                        });

                        s.call(req).await
                    } else {
                        Ok(req.into_response(log_out_user()))
                    }
                } else {
                    Ok(req.into_response(log_out_user()))
                }
            } else {
                Ok(s.call(req).await?)
            }
        }
        .instrument(tracing::span::Span::current())
        .boxed_local()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::{LOCATION, SET_COOKIE};
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App, HttpRequest};

    /// A pool that never connects (port 1). Only the branches that don't touch
    /// the DB are exercised here; anything that calls `pool.get()` would fail.
    fn dead_pool() -> Pool<PostgresConnectionManager<NoTls>> {
        let manager = PostgresConnectionManager::new(
            "host=127.0.0.1 port=1 user=x dbname=x".parse().unwrap(),
            NoTls,
        );
        Pool::builder().build_unchecked(manager)
    }

    async fn echo(req: HttpRequest) -> HttpResponse {
        let has_user = req.extensions().get::<UserSession>().is_some();
        HttpResponse::Ok().body(if has_user { "user" } else { "anon" })
    }

    async fn drive(
        req: test::TestRequest,
    ) -> ServiceResponse<EitherBody<actix_web::body::BoxBody>> {
        // UserSessionTransformer requires the service it wraps to already yield an
        // `EitherBody` (so it can short-circuit with a logout response). In the
        // real app another layer provides that; here a tiny inner `wrap_fn` maps
        // the handler's BoxBody into the left `EitherBody` variant.
        // The first `.wrap()` is innermost: the body-mapper wraps routing and
        // converts BoxBody → EitherBody, then UserSessionTransformer wraps that.
        let app = test::init_service(
            App::new()
                .app_data(Data::new(dead_pool()))
                .wrap_fn(|req, srv| {
                    let fut = srv.call(req);
                    async move {
                        fut.await
                            .map(ServiceResponse::map_into_left_body::<actix_web::body::BoxBody>)
                    }
                })
                .wrap(UserSessionTransformer)
                .default_service(web::to(echo)),
        )
        .await;
        test::call_service(&app, req.to_request()).await
    }

    #[actix_web::test]
    async fn passes_through_when_no_auth_cookies() {
        // No username cookie → the middleware returns early (before ever touching
        // the pool), so the dead pool is never queried.
        let resp = drive(test::TestRequest::get()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        assert_eq!(&body[..], b"anon");
    }

    #[actix_web::test]
    async fn username_without_token_logs_out_and_clears_cookies() {
        // A `username` cookie without a `token` is a stale session → 301 logout
        // that clears both cookies. This DB-free branch also exercises the
        // logout-cookie clearing, and the axum rewrite asserts the identical
        // behavior — the refactor must preserve it.
        let resp = drive(
            test::TestRequest::get().cookie(actix_web::cookie::Cookie::new("username", "neo")),
        )
        .await;

        assert_eq!(resp.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(resp.headers().get(LOCATION).unwrap(), "/");

        let cookies: Vec<String> = resp
            .headers()
            .get_all(SET_COOKIE)
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        assert_eq!(cookies.len(), 2);
        for name in ["username", "token"] {
            assert!(
                cookies
                    .iter()
                    .any(|c| c.starts_with(&format!("{name}=")) && c.contains("Expires=")),
                "expected a cleared {name} cookie, got {cookies:?}"
            );
        }
    }
}
