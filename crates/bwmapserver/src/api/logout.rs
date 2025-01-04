use actix_web::{cookie::Cookie, get, HttpResponse, Responder};

async fn handler2() -> Result<impl Responder, bwcommon::MyError> {
    let info = bwcommon::ApiSpecificInfoForLogging {
        ..Default::default()
    };

    Ok(
        bwcommon::insert_extension(HttpResponse::TemporaryRedirect(), info)
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
            .customize()
            .insert_header(("location", "/")),
    )
}

// Ok(insert_extension(HttpResponse::TemporaryRedirect(), info)
// .header("Location", format!("/map/{}", map_id))
// .finish())

#[get("/api/logout")]
async fn handler() -> Result<impl Responder, bwcommon::MyError> {
    handler2().await
}
