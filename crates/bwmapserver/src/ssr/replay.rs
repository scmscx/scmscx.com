use crate::middleware::UserSession;
use crate::ssr::get_navbar_langmap;
use actix_web::{get, web, HttpMessage, HttpResponse, Responder};
use actix_web::{HttpRequest, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[get("/uiv1/replay/{replay_id}")]
async fn handler(
    req: HttpRequest,
    path: web::Path<(i64,)>,
    hb: web::Data<handlebars::Handlebars<'_>>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
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

    let (replay_id,) = path.into_inner();

    let (uploaded_by, uploaded_time, replay_blob, denorm_scenario, chkhash, map_id) = {
        let r = pool.get().await?.query_one("
                select account.username, replay.uploaded_time, replayblob.data, map.denorm_scenario, replay.chkhash, map.id
                from replay
                join replayblob on replayblob.hash = replay.hash
                join account on account.id = uploaded_by
                full outer join map on map.chkblob = replay.chkhash
                where replay.id = $1", &[&replay_id]).await?;

        (
            r.try_get::<_, String>(0)?,
            r.try_get::<_, i64>(1)?,
            r.try_get::<_, Vec<u8>>(2)?,
            r.try_get::<_, String>(3)?,
            r.try_get::<_, String>(4)?,
            r.try_get::<_, i64>(5)?,
        )
    };

    #[derive(Debug, Serialize, Deserialize)]
    struct ReplayInfo {
        uploaded_by: String,
        uploaded_time: i64,
        replay_header: bwreplay::ReplayHeader,
        denorm_scenario: String,
        chkhash: String,
        map_id: i64,
    }

    let ret = ReplayInfo {
        uploaded_by,
        uploaded_time,
        replay_header: bwreplay::parse_replay_blob(replay_blob.as_slice())?.header,
        denorm_scenario,
        chkhash,
        map_id,
    };

    // ret.replay_header.slots_players[0].

    let players: Vec<_> = ret
        .replay_header
        .slots_players
        .iter()
        .map(|x| {
            json!({
                "player_id": x.player_id,
                "player_name": std::str::from_utf8(x.player_name.as_slice()).unwrap_or("Couldn't decode utf8 player name"),
                "player_race": x.player_race,
                "player_team": x.player_team,
                "player_type": x.player_type,
                "slot_id": x.slot_id,
            })
        })
        .collect();

    // ret.replay_header
    Ok(HttpResponse::Ok().content_type("text/html").body(hb.render(
        "replay",
        &json!({
            "scenario_name": ret.denorm_scenario,
            "chkhash": ret.chkhash,
            "map_id": ret.map_id,
            "uploaded_time": ret.uploaded_time,
            "uploaded_by": ret.uploaded_by,
            "frames": ret.replay_header.frames,
            "time_save": ret.replay_header.time_save,
            "height": ret.replay_header.height,
            "width": ret.replay_header.width,
            "game_name": std::str::from_utf8(ret.replay_header.name_game.as_slice()).unwrap_or("Couldn't utf8 decode game name"),
            "gamespeed": ret.replay_header.speed,
            "game_type": ret.replay_header.game_type,
            "game_subtype": ret.replay_header.game_subtype,
            "engine": ret.replay_header.engine,
            "available_slots": ret.replay_header.count_available_slots,
            "lobby_creator": std::str::from_utf8(ret.replay_header.name_creator.as_slice()).unwrap_or("Couldn't utf8 decode creator name"),
            "players": players,
            "langmap": json!({ "navbar": get_navbar_langmap(lang) }),
            "is_logged_in": user_username.1,
            "username": user_username.0,
        }),
    )?))
}
