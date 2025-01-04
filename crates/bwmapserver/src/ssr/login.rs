use crate::middleware::UserSession;
use crate::ssr::get_navbar_langmap;
use actix_web::{get, web, HttpMessage, HttpResponse, Responder};
use actix_web::{HttpRequest, Result};
use serde_json::json;

#[get("/uiv1/login")]
async fn handler(
    req: HttpRequest,
    hb: web::Data<handlebars::Handlebars<'_>>,
) -> Result<impl Responder, bwcommon::MyError> {
    let lang = req
        .extensions()
        .get::<bwcommon::LangData>()
        .unwrap_or(&bwcommon::LangData::English)
        .to_owned();

    let user_username = req
        .extensions()
        .get::<UserSession>()
        .map(|x| (x.username.clone(), true))
        .unwrap_or_default();

    Ok(HttpResponse::Ok().content_type("text/html").body(hb.render(
        "login",
        &json!({
            "langmap": json!({ "navbar": get_navbar_langmap(lang) }),
            "is_logged_in": user_username.1,
            "username": user_username.0,
        }),
    )?))
}
