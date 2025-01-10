use crate::util::is_dev_mode;
use actix_web::{cookie::Cookie, post, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginFormData {
    username: String,
    password: String,
}

async fn handler2(
    form: web::Json<LoginFormData>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<HttpResponse, bwcommon::MyError> {
    if let Ok(token) = crate::db::login(form.username.clone(), form.password.clone(), pool).await {
        let info = bwcommon::ApiSpecificInfoForLogging {
            username: Some(form.username.clone()),
            ..Default::default()
        };

        Ok(bwcommon::insert_extension(HttpResponse::Ok(), info)
            .cookie(
                Cookie::build("token", token)
                    .path("/")
                    .same_site(actix_web::cookie::SameSite::Lax)
                    .secure(!is_dev_mode())
                    .permanent()
                    .http_only(true)
                    .finish(),
            )
            .cookie(
                Cookie::build("username", &form.username)
                    .path("/")
                    .same_site(actix_web::cookie::SameSite::Lax)
                    .secure(!is_dev_mode())
                    .permanent()
                    .finish(),
            )
            .finish())
    } else {
        Ok(HttpResponse::Unauthorized()
            .body("Either the username does not exist or the password is incorrect."))
    }
}

#[post("/api/login")]
async fn post_handler(
    form: web::Json<LoginFormData>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<HttpResponse, bwcommon::MyError> {
    handler2(form, pool).await
}
