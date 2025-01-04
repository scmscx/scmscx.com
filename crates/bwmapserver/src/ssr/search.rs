use crate::middleware::UserSession;
use crate::search2::{search, SearchParams};
use crate::ssr::get_navbar_langmap;
use actix_web::HttpMessage;
use actix_web::{get, web, HttpResponse, Responder};
use actix_web::{HttpRequest, Result};
use tracing::instrument;

// trait MyTrait: r2d2_postgres::postgres::types::ToSql + Sync + std::fmt::Display {
//     fn to_sub(&self) -> &(dyn r2d2_postgres::postgres::types::ToSql + Sync);
// }

// impl<T: r2d2_postgres::postgres::types::ToSql + Sync + std::fmt::Display> MyTrait for T {
//     fn to_sub(&self) -> &(dyn r2d2_postgres::postgres::types::ToSql + Sync) {
//         self
//     }
// }

// trait SuperCached {
//     fn cached_query<F, T, E>(
//         &mut self,
//         query: &str,
//         params: &[&dyn MyTrait],
//         cache: &std::sync::Mutex<lru::LruCache<String, Vec<T>>>,
//         map: F,
//     ) -> Result<Vec<T>, E>
//     where
//         F: FnMut(r2d2_postgres::postgres::row::Row) -> Result<T, E>,
//         T: Clone,
//         E: From<r2d2_postgres::postgres::Error>;
// }

// impl SuperCached
//     for r2d2_postgres::r2d2::PooledConnection<
//         r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>,
//     >
// {
//     fn cached_query<F, T, E>(
//         &mut self,
//         query: &str,
//         params: &[&dyn MyTrait],
//         cache: &std::sync::Mutex<lru::LruCache<String, Vec<T>>>,
//         map: F,
//     ) -> Result<Vec<T>, E>
//     where
//         F: FnMut(r2d2_postgres::postgres::row::Row) -> Result<T, E>,
//         T: Clone,
//         E: From<r2d2_postgres::postgres::Error>,
//     {
//         let mut key = format!("{}", query);

//         for v in params {
//             key = format!("{}+{}", key, v);
//         }

//         {
//             let mut guard = cache.lock().unwrap();
//             if let Some(cached_value) = guard.get(&key.to_string()) {
//                 return Ok(cached_value.clone());
//             }
//         }

//         let mut v: Vec<&(dyn r2d2_postgres::postgres::types::ToSql + Sync)> = Vec::new();
//         for k in params {
//             v.push(k.to_sub());
//         }
//         let vec = self
//             .query(query, v.as_slice())?
//             .into_iter()
//             .map(map)
//             .collect::<Result<Vec<T>, E>>()?;

//         if let Ok(mut guard) = cache.lock() {
//             guard.put(key, vec.clone());
//         }

//         Ok(vec)
//     }
// }

#[instrument(skip_all, name = "/search")]
async fn handler2(
    req: HttpRequest,
    query: String,
    query_params: web::Query<SearchParams>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
) -> Result<impl Responder, bwcommon::MyError> {
    // lazy_static::lazy_static! {
    //     static ref CACHE: std::sync::Mutex<lru::LruCache::<String, Vec<Map>>> = {
    //         std::sync::Mutex::new(lru::LruCache::<String, Vec<Map>>::new(100))
    //     };
    // }

    // let needs_drop = {
    //     let mut guard = cache_droper.lock().unwrap();
    //     guard.insert(format!("{}{}", file!(), line!()))
    // };

    // if needs_drop {
    //     CACHE.lock().unwrap().clear();
    // }

    // cache_droper: actix_web::web::Data<std::sync::Mutex<std::collections::HashSet<String>>>,

    let allow_nsfw = bwcommon::check_auth4(&req, (**pool).clone())
        .await?
        .is_some();

    let maps = search(query.as_str(), allow_nsfw, &query_params, pool.clone()).await?;

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

    let page_title = if query.is_empty() {
        "Search StarCraft: Brood War Maps".to_owned()
    } else {
        format!("{} maps found for: {}", maps.len(), query)
    };

    let langmap = if lang == bwcommon::LangData::Korean {
        serde_json::json!({
            "h1": "데이터베이스의 100,000개 지도에서 지도 이름, 부대 이름, 설명 및 부대 이름 검색",
            "h4_try_popular_searches": "인기 검색어 시도",
            "h4_did_you_make_maps": "지도를 만드셨나요? 사용한 이름을 검색해 보세요",
            "random_button": "무작위의",
            "search_button": "검색",
            "unit_names": "단위 이름",
            "force_names": "포스 이름",
            "file_names": "파일 이름",
            "scenario_names": "시나리오 이름",
            "scenario_descriptions": "시나리오 설명",
            "results": "결과",
            "scenario": "대본",
            "last_modified_time": "마지막 수정 시간",
            "uploaded_time": "업로드 시간",
            "navbar": get_navbar_langmap(lang),
        })
    } else {
        serde_json::json!({
            "h1": "Search map names, unit names, descriptions, and force names across over 100,000 maps in the database",
            "h4_try_popular_searches": "Try Popular searches",
            "h4_did_you_make_maps": "Did you make maps? Try searching the name you used",
            "random_button": "Random",
            "search_button": "Search",
            "unit_names": "Unit Names",
            "force_names": "Force Names",
            "file_names": "File Names",
            "scenario_names": "Scenario Names",
            "scenario_descriptions": "Scenario Descriptions",
            "results": "Results",
            "scenario": "Scenario",
            "last_modified_time": "Last Modified Time",
            "uploaded_time": "Uploaded Time",
            "navbar": get_navbar_langmap(lang),
        })
    };

    let new_html = hb.render(
        "search",
        &serde_json::json!({
            "page_title": page_title,
            "search_results": serde_json::to_string(&maps)?,
            "langmap": langmap,
            "is_logged_in": user_username.1,
            "username": user_username.0,
        }),
    )?;

    Ok(HttpResponse::Ok().content_type("text/html").body(new_html))
}

#[get("/uiv1/search/{query}")]
async fn handler(
    req: HttpRequest,
    path: web::Path<(String,)>,
    query_params: web::Query<SearchParams>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
) -> Result<impl Responder, bwcommon::MyError> {
    let (query,) = path.into_inner();
    handler2(req, query, query_params, pool, hb).await
}

#[get("/uiv1/search")]
async fn handler_empty_query(
    req: HttpRequest,
    query_params: web::Query<SearchParams>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
    hb: web::Data<handlebars::Handlebars<'_>>,
) -> Result<impl Responder, bwcommon::MyError> {
    handler2(req, "".to_string(), query_params, pool, hb).await
}
