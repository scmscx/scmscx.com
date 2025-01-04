use actix_web::{post, web, HttpRequest, HttpResponse};

#[derive(serde::Deserialize)]
struct ChangePasswordPostData {
    password: String,
    password_confirm: String,
}

async fn handler2(
    req: HttpRequest,
    form: web::Json<ChangePasswordPostData>,
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

    crate::db::change_password(user_id, form.password.clone(), pool).await?;

    let info = bwcommon::ApiSpecificInfoForLogging {
        user_id: Some(user_id),
        ..Default::default()
    };

    Ok(bwcommon::insert_extension(HttpResponse::Ok(), info).body("Password changed successfully"))
}

#[post("/api/change-password")]
async fn post_handler(
    req: HttpRequest,
    form: web::Json<ChangePasswordPostData>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<HttpResponse, bwcommon::MyError> {
    handler2(req, form, pool).await
}
