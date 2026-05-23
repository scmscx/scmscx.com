use crate::ratelimit::UsernameLoginLimiter;
use crate::util::is_dev_mode;
use actix_web::{cookie::Cookie, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct LoginFormData {
    username: String,
    password: String,
}

pub async fn post_handler(
    form: web::Json<LoginFormData>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    username_limiter: web::Data<UsernameLoginLimiter>,
) -> Result<HttpResponse, bwcommon::MyError> {
    if form.username.is_empty() || form.username.len() > 100 {
        return Ok(HttpResponse::Unauthorized()
            .body("Either the username does not exist or the password is incorrect."));
    }

    // Username only appears in the JSON body, so this can't be enforced as middleware.
    if let Err(resp) = username_limiter.check(&form.username) {
        return Ok(resp);
    }

    match crate::db::login(form.username.clone(), form.password.clone(), pool).await {
        Ok(token) => {
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
        }
        Err(_) => Ok(HttpResponse::Unauthorized()
            .body("Either the username does not exist or the password is incorrect.")),
    }
}
