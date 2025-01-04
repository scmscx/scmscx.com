use actix_web::{cookie::Cookie, post, web, HttpRequest, HttpResponse};

#[derive(serde::Deserialize)]
struct ChangeUsernamePostData {
    username: String,
    username_confirm: String,
    password: String,
}

async fn handler2(
    req: HttpRequest,
    form: web::Json<ChangeUsernamePostData>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<HttpResponse, bwcommon::MyError> {
    let user_id = if let Some(user_id) = bwcommon::check_auth4(&req, (**pool).clone()).await? {
        user_id
    } else {
        return Ok(HttpResponse::Unauthorized().body("Unauthorized. Try logging in first/again."));
    };

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

    if form.username != form.username_confirm {
        return Ok(HttpResponse::BadRequest().body("The provided usernames must match"));
    }

    let is_password_correct =
        crate::db::check_password(user_id, form.password.clone(), pool.clone()).await?;

    if !is_password_correct {
        return Ok(HttpResponse::BadRequest().body("Provided password is incorrect."));
    }

    crate::db::change_username(user_id, form.username.clone(), form.password.clone(), pool).await?;

    let info = bwcommon::ApiSpecificInfoForLogging {
        user_id: Some(user_id),
        ..Default::default()
    };

    Ok(bwcommon::insert_extension(HttpResponse::Ok(), info)
        .cookie(
            Cookie::build("username", &form.username)
                .path("/")
                .same_site(actix_web::cookie::SameSite::Lax)
                .secure(true)
                .permanent()
                .finish(),
        )
        .body("Username changed successfully"))
}

#[post("/api/change-username")]
async fn post_handler(
    req: HttpRequest,
    form: web::Json<ChangeUsernamePostData>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<HttpResponse, bwcommon::MyError> {
    handler2(req, form, pool).await
}
