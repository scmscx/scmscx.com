use crate::middleware::UserSession;
use crate::ssr::get_navbar_langmap;
use actix_web::{get, web, HttpMessage, HttpResponse, Responder};
use actix_web::{HttpRequest, Result};
use serde_json::json;

#[get("/uiv1/upload-map")]
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

    let langmap = if lang == bwcommon::LangData::Korean {
        serde_json::json!({
            "h1": "업로드",
            "h2": ".scm/.scx 파일 또는 맵 파일의 전체 디렉토리 업로드",
            "h4_p1": "하나 이상의 .scm/.scx 파일을 업로드하려면 상단 파일 선택기를 선택하십시오. 전체 디렉터리와 해당 하위 디렉터리를 업로드하려면 맨 아래 파일 선택기를 선택합니다.",
            "h4_p2": "또한 손상, 파손, 미완성, 테스트, 조작, 보호되지 않음, 복제 또는
            오래된 지도.
            그만큼
            웹사이트가 이 모든 것을 처리할 것이며 그 중 많은 부분이 스타크래프트 지도 제작 역사의 중요한 부분입니다. 조차
            정확히 동일한 아카이브를 여러 번 업로드하는 것은 문제가 되지 않습니다. 그러니 가지고 있는 모든 것을 업로드하고
            사이트에서 필터링 및 처리를 수행합니다. 전체 StarCraft 지도 디렉토리를 업로드해 보십시오: <USER_HOME>\\My Documents\\StarCraft",
            "archive_label": "보관소",
            "upload_button": "업로드",
            "navbar": get_navbar_langmap(lang),
        })
    } else {
        serde_json::json!({
            "h1": "Upload",
            "h2": "Upload .scm/.scx files or entire directories of map files",
            "h4_p1": "If you want to upload one or more .scm/.scx files, then choose the top file picker. If you want to upload entire directories and their sub directories, then choose the bottom file picker.",
            "h4_p2": "Also, please don't worry about corrupt, broken, unfinished, testing, rigged, unprotected, duplicate or
            outdated maps.
            The
            website will handle all of this and many of them are important parts of StarCraft map making history. Even
            uploading the exact same archive multiple times is no concern. So, please upload everything you have and let
            the site do the filtering and processing. Try uploading your entire StarCraft map directory: <USER_HOME>\\My Documents\\StarCraft",
            "archive_label": "Archive",
            "upload_button": "Upload",
            "navbar": get_navbar_langmap(lang),
        })
    };

    Ok(HttpResponse::Ok().content_type("text/html").body(hb.render(
        "upload-map",
        &json!({
            "langmap": langmap,
            "is_logged_in": user_username.1,
            "username": user_username.0,
        }),
    )?))
}
