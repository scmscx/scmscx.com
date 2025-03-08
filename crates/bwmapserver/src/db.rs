use actix_web::web;
use sha2::{Digest, Sha256};

pub(crate) async fn get_chk(
    chk_id: String,
    pool: bb8_postgres::bb8::Pool<
        bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
    >,
) -> Result<Vec<u8>, anyhow::Error> {
    let con = pool.get().await?;
    let row = con
        .query_one(
            "
            select length, ver, data
            from chkblob
            where hash = $1",
            &[&chk_id],
        )
        .await?;

    let length = row.try_get::<_, i64>("length")? as usize;
    let ver = row.try_get::<_, i64>("ver")?;
    let data = row.try_get::<_, Vec<u8>>("data")?;

    anyhow::ensure!(ver == 1);

    anyhow::Ok(zstd::bulk::decompress(data.as_slice(), length)?)
}

pub(crate) async fn get_minimap(
    chk_id: String,
    pool: bb8_postgres::bb8::Pool<
        bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
    >,
) -> Result<(i32, i32, Vec<u8>), anyhow::Error> {
    let con = pool.get().await?;
    let row = con
        .query_one(
            "
            select width, height, minimap
            from minimap
            where chkhash = $1",
            &[&chk_id],
        )
        .await?;

    anyhow::Ok((
        row.try_get::<_, i32>("width")?,
        row.try_get::<_, i32>("height")?,
        row.try_get::<_, Vec<u8>>("minimap")?,
    ))
}

pub(crate) async fn change_password(
    user_id: i64,
    password: String,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<(), anyhow::Error> {
    let con = pool.get().await?;
    let username = con
        .query_one(
            "
            select username from account where id = $1",
            &[&user_id],
        )
        .await?
        .try_get::<_, String>("username")?;

    let salt = uuid::Uuid::new_v4().as_simple().to_string();

    let hashed_password = {
        let mut hasher = Sha256::new();
        hasher.update(&username.as_bytes());
        hasher.update(&password.as_bytes());
        hasher.update(&salt.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    con.execute(
        "UPDATE account set passwordhash = $1, salt = $2 where id = $3",
        &[&hashed_password, &salt, &user_id],
    )
    .await?;
    anyhow::Ok(())
}

pub(crate) async fn check_password(
    user_id: i64,
    password: String,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<bool, anyhow::Error> {
    let con = pool.get().await?;
    let row = con
        .query_one(
            "select username, salt from account where id = $1",
            &[&user_id],
        )
        .await?;
    let username = row.try_get::<_, String>("username")?;
    let salt = row.try_get::<_, String>("salt")?;

    let hashed_provided_password = {
        let mut hasher = Sha256::new();
        hasher.update(&username.as_bytes());
        hasher.update(&password.as_bytes());
        hasher.update(&salt.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let hashed_existing_password: String = con
        .query_one(
            "select passwordhash from account where id = $1",
            &[&user_id],
        )
        .await?
        .try_get(0)?;

    anyhow::Ok(hashed_provided_password == hashed_existing_password)
}

pub(crate) async fn change_username(
    user_id: i64,
    new_username: String,
    password: String,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<(), anyhow::Error> {
    let con = pool.get().await?;

    let salt = uuid::Uuid::new_v4().as_simple().to_string();

    let hashed_password = {
        let mut hasher = Sha256::new();
        hasher.update(&new_username.as_bytes());
        hasher.update(&password.as_bytes());
        hasher.update(&salt.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    con.execute(
        "UPDATE account set username = $1, passwordhash = $2, salt = $3 where id = $4",
        &[&new_username, &hashed_password, &salt, &user_id],
    )
    .await?;
    anyhow::Ok(())
}

pub(crate) async fn set_tags(
    map_id: i64,
    map: std::collections::HashMap<String, String>,
    user_id: Option<i64>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<(), anyhow::Error> {
    let mut con = pool.get().await?;
    let tx = con.transaction().await?;

    let map_uploader: i64 = tx
        .query_one("select uploaded_by from map where map.id = $1", &[&map_id])
        .await?
        .try_get(0)?;

    if let Some(id) = user_id {
        if map_uploader != id && id != 4 {
            return Ok(());
        }
    }
    tx.execute("delete from tagmap where tagmap.map = $1", &[&map_id])
        .await?;

    for t in map {
        let tag_id = tx
            .query_one(
                "insert into tag (key, value) values ($1, $2) RETURNING id",
                &[&t.0, &t.1],
            )
            .await?
            .try_get::<_, i64>(0)?;

        tx.execute(
            "insert into tagmap (map, tag) values ($1, $2)",
            &[&map_id, &tag_id],
        )
        .await?;
    }
    tx.commit().await?;
    anyhow::Ok(())
}

pub(crate) async fn add_tags(
    map_id: i64,
    map: std::collections::HashMap<String, String>,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<(), anyhow::Error> {
    let mut con = pool.get().await?;
    let tx = con.transaction().await?;

    for t in map {
        let tag_id = tx
            .query_one(
                "insert into tag (key, value) values ($1, $2) RETURNING id",
                &[&t.0, &t.1],
            )
            .await?
            .try_get::<_, i64>(0)?;

        tx.execute(
            "insert into tagmap (map, tag) values ($1, $2)",
            &[&map_id, &tag_id],
        )
        .await?;
    }
    tx.commit().await?;
    anyhow::Ok(())
}

pub(crate) async fn login(
    username: String,
    password: String,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<String, anyhow::Error> {
    let con = pool.get().await?;

    let salt = con
        .query_one("select salt from account where username = $1", &[&username])
        .await?
        .try_get::<_, String>(0)?;

    let hashed_password = {
        let mut hasher = Sha256::new();
        hasher.update(&username.as_bytes());
        hasher.update(&password.as_bytes());
        hasher.update(&salt.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let token = con
        .query_one(
            "select token from account where username = $1 and passwordhash = $2",
            &[&username, &hashed_password],
        )
        .await?
        .try_get(0)?;

    anyhow::Ok(token)
}

pub(crate) async fn register(
    username: String,
    password: String,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<String, anyhow::Error> {
    let con = pool.get().await?;

    let salt = uuid::Uuid::new_v4().as_simple().to_string();

    let hashed_password = {
        let mut hasher = Sha256::new();
        hasher.update(&username.as_bytes());
        hasher.update(&password.as_bytes());
        hasher.update(&salt.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let token = uuid::Uuid::new_v4().as_simple().to_string();

    let rows_updated = con
        .execute(
            "insert into account (username, passwordhash, salt, token) values ($1, $2, $3, $4)",
            &[&username, &hashed_password, &salt, &token],
        )
        .await?;

    if rows_updated == 1 {
        anyhow::Ok(token)
    } else {
        Err(anyhow::anyhow!("username already exists"))
    }
}

// pub(crate) async fn insert_replay(
//     replay_blob: &[u8],
//     user_id: i64,
//     pool: web::Data<
//         bb8_postgres::bb8::Pool<
//             bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
//         >,
//     >,
// ) -> Result<i64, anyhow::Error> {
//     let replay_blob = replay_blob.to_vec();

//     let replay_hash = {
//         let mut hasher = Sha256::new();
//         hasher.update(&replay_blob);
//         format!("{:x}", hasher.finalize())
//     };

//     // parse replay and denorm
//     let parsed_replay = bwreplay::parse_replay_blob(replay_blob.as_slice())?;

//     let chk_blob = parsed_replay.chk_data;
//     let chk_blob_hash = {
//         let mut hasher = Sha256::new();
//         hasher.update(&chk_blob);
//         format!("{:x}", hasher.finalize())
//     };
//     let compressed_chk = zstd::bulk::compress(chk_blob.as_slice(), 15)?;

//     let denorm_game_creator = parsed_replay.header.name_creator;
//     let denorm_time_saved = parsed_replay.header.time_save as i64;
//     let denorm_frames = parsed_replay.header.frames as i64;
//     let denorm_number_of_human_players = parsed_replay.header.slots_players.len() as i64;
//     let denorm_first_human_player = parsed_replay
//         .header
//         .slots_players
//         .first()
//         .ok_or_else(|| anyhow::anyhow!("0 player replay..?"))?
//         .player_name
//         .clone();
//     let denorm_scenario = parsed_replay.header.name_scenario;
//     let denorm_game = parsed_replay.header.name_game;

//     // get current time
//     let time_since_epoch = std::time::SystemTime::now()
//         .duration_since(std::time::UNIX_EPOCH)?
//         .as_secs() as i64;

//     // db stuff
//     let mut con = pool.get().await?;

//     let tx = con.transaction().await?;

//     tx.execute(
//         "insert into replayblob (hash, data) values ($1, $2) ON CONFLICT DO NOTHING",
//         &[&replay_hash, &replay_blob],
//     )
//     .await?;
//     tx.execute("insert into chkblob (hash, ver, length, data) values ($1, 1, $2, $3) ON CONFLICT DO NOTHING", &[&chk_blob_hash, &(chk_blob.len() as i64), &compressed_chk]).await?;

//     tx.execute("
//         insert into replay
//         (hash, uploaded_by, uploaded_time, denorm_game_creator, denorm_time_saved, denorm_frames, denorm_number_of_human_players, denorm_first_human_player, denorm_scenario, denorm_game, chkhash)
//         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) ON CONFLICT DO NOTHING",
//         &[&replay_hash, &user_id, &time_since_epoch, &denorm_game_creator, &denorm_time_saved, &denorm_frames, &denorm_number_of_human_players, &denorm_first_human_player, &denorm_scenario, &denorm_game, &chk_blob_hash]).await?;

//     let replay_id = tx
//         .query_one("select id from replay where hash = $1", &[&replay_hash])
//         .await?
//         .try_get::<_, i64>(0)?;

//     tx.commit().await?;

//     anyhow::Ok(replay_id)
// }
