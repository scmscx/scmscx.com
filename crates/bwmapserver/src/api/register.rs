use actix_web::{
    cookie::{Cookie, SameSite},
    post, web, HttpResponse,
};

#[derive(serde::Deserialize)]
struct RegisterFormData {
    username: String,
    password: String,
    password_confirm: String,
}

async fn handler2(
    form: web::Json<RegisterFormData>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<HttpResponse, bwcommon::MyError> {
    if form.username.len() < 1 {
        return Ok(
            HttpResponse::BadRequest().body("The provided username must not be the empty string")
        );
    }

    if form.username.len() > 100 {
        return Ok(
            HttpResponse::BadRequest().body("Why would you try to create a username that long")
        );
    }

    if form.password != form.password_confirm {
        return Ok(HttpResponse::BadRequest().body("The two provided passwords must match"));
    }

    if form.password.len() < 1 {
        return Ok(
            HttpResponse::BadRequest().body("The provided password must not be the empty string")
        );
    }

    if form.password.len() > 100 {
        return Ok(
            HttpResponse::BadRequest().body("Why would you try to create a password that long")
        );
    }

    if let Ok(token) = crate::db::register(form.username.clone(), form.password.clone(), pool).await
    {
        let info = bwcommon::ApiSpecificInfoForLogging {
            username: Some(form.username.clone()),
            ..Default::default()
        };

        Ok(bwcommon::insert_extension(HttpResponse::Ok(), info)
            .cookie(
                Cookie::build("token", token)
                    .path("/")
                    .same_site(SameSite::Lax)
                    .secure(true)
                    .permanent()
                    .http_only(true)
                    .finish(),
            )
            .cookie(
                Cookie::build("username", &form.username)
                    .path("/")
                    .same_site(SameSite::Lax)
                    .secure(true)
                    .permanent()
                    .finish(),
            )
            .finish())
    } else {
        Ok(HttpResponse::Unauthorized().body("Could not register account"))
    }
}

#[post("/api/register")]
async fn post_handler(
    form: web::Json<RegisterFormData>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<HttpResponse, bwcommon::MyError> {
    handler2(form, pool).await
}
