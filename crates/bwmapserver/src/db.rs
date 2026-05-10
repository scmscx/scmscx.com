use actix_web::web;
use argon2::Argon2;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine as _;
use rand::RngCore;
use sha2::{Digest, Sha256};
use tracing::warn;

const ARGON2_SALT_LEN: usize = 16;
const ARGON2_HASH_LEN: usize = 32;

pub(crate) const PASSWORD_ALGO_ARGON2ID: &str = "argon2id";
pub(crate) const PASSWORD_ALGO_SHA256_LEGACY: &str = "sha256-legacy";

pub(crate) struct HashedPassword {
    pub algorithm: &'static str,
    pub salt: String,
    pub hash: String,
}

/// Run Argon2id with default OWASP params for the given password + salt.
fn argon2id_kdf(password: &[u8], salt: &[u8]) -> Result<[u8; ARGON2_HASH_LEN], anyhow::Error> {
    let mut out = [0u8; ARGON2_HASH_LEN];
    Argon2::default()
        .hash_password_into(password, salt, &mut out)
        .map_err(|e| anyhow::anyhow!("argon2 hash failed: {e}"))?;
    Ok(out)
}

/// Hash a password with Argon2id and a fresh random salt. Returns the
/// algorithm label plus base64-encoded salt and hash bytes, ready to write
/// into the `password_algorithm`, `salt`, and `passwordhash` columns.
fn hash_password(password: &str) -> Result<HashedPassword, anyhow::Error> {
    let mut salt_bytes = [0u8; ARGON2_SALT_LEN];
    rand::rng().fill_bytes(&mut salt_bytes);

    let hash_bytes = argon2id_kdf(password.as_bytes(), &salt_bytes)?;

    Ok(HashedPassword {
        algorithm: PASSWORD_ALGO_ARGON2ID,
        salt: STANDARD_NO_PAD.encode(salt_bytes),
        hash: STANDARD_NO_PAD.encode(hash_bytes),
    })
}

/// Verify `password` against the stored hash, dispatching on `algorithm`
/// (the `password_algorithm` column). `username` is only used by the
/// legacy SHA-256 path.
fn verify_password(
    algorithm: &str,
    password: &str,
    username: &str,
    salt: &str,
    stored_hash: &str,
) -> bool {
    match algorithm {
        PASSWORD_ALGO_ARGON2ID => {
            let Ok(salt_bytes) = STANDARD_NO_PAD.decode(salt) else {
                return false;
            };
            let Ok(expected) = STANDARD_NO_PAD.decode(stored_hash) else {
                return false;
            };
            let Ok(computed) = argon2id_kdf(password.as_bytes(), &salt_bytes) else {
                return false;
            };
            ct_eq(&computed, &expected)
        }
        PASSWORD_ALGO_SHA256_LEGACY => {
            let computed = {
                let mut hasher = Sha256::new();
                hasher.update(username.as_bytes());
                hasher.update(password.as_bytes());
                hasher.update(salt.as_bytes());
                format!("{:x}", hasher.finalize())
            };
            ct_eq(computed.as_bytes(), stored_hash.as_bytes())
        }
        _ => false,
    }
}

/// Constant-time byte slice equality. Avoids early-exit timing leaks that
/// `==` on slices may have.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

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
    let new = hash_password(&password)?;
    con.execute(
        "UPDATE account set password_algorithm = $1, passwordhash = $2, salt = $3 where id = $4",
        &[&new.algorithm, &new.hash, &new.salt, &user_id],
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
            "select username, password_algorithm, salt, passwordhash from account where id = $1",
            &[&user_id],
        )
        .await?;
    let username: String = row.try_get("username")?;
    let algorithm: String = row.try_get("password_algorithm")?;
    let salt: String = row.try_get("salt")?;
    let stored_hash: String = row.try_get("passwordhash")?;

    anyhow::Ok(verify_password(
        &algorithm,
        &password,
        &username,
        &salt,
        &stored_hash,
    ))
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
    // Legacy SHA-256 hashes mix the username into the digest, so changing
    // the username invalidates them. Re-hashing with Argon2id here covers
    // legacy users and opportunistically migrates remaining entries.
    let new = hash_password(&password)?;
    con.execute(
        "UPDATE account set username = $1, password_algorithm = $2, passwordhash = $3, salt = $4 where id = $5",
        &[&new_username, &new.algorithm, &new.hash, &new.salt, &user_id],
    )
    .await?;
    anyhow::Ok(())
}

pub(crate) async fn set_tags(
    map_id: i64,
    map: std::collections::HashMap<String, String>,
    user_id: i64,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<Option<bool>, anyhow::Error> {
    let mut con = pool.get().await?;
    let tx = con.transaction().await?;

    let Some(row) = tx
        .query_opt("select uploaded_by from map where map.id = $1", &[&map_id])
        .await?
    else {
        return anyhow::Ok(None);
    };
    let map_uploader: i64 = row.try_get(0)?;

    if map_uploader != user_id && user_id != 4 {
        return anyhow::Ok(Some(false));
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
    anyhow::Ok(Some(true))
}

pub(crate) async fn add_tags(
    map_id: i64,
    map: std::collections::HashMap<String, String>,
    user_id: i64,
    pool: web::Data<
        bb8_postgres::bb8::Pool<
            bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
        >,
    >,
) -> Result<Option<bool>, anyhow::Error> {
    let mut con = pool.get().await?;
    let tx = con.transaction().await?;

    let Some(row) = tx
        .query_opt("select uploaded_by from map where map.id = $1", &[&map_id])
        .await?
    else {
        return anyhow::Ok(None);
    };
    let map_uploader: i64 = row.try_get(0)?;

    if map_uploader != user_id && user_id != 4 {
        return anyhow::Ok(Some(false));
    }

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
    anyhow::Ok(Some(true))
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

    let Some(row) = con
        .query_opt(
            "select id, password_algorithm, passwordhash, salt, token from account where username = $1",
            &[&username],
        )
        .await?
    else {
        return Err(anyhow::anyhow!("invalid credentials"));
    };

    let user_id: i64 = row.try_get("id")?;
    let algorithm: String = row.try_get("password_algorithm")?;
    let stored_hash: String = row.try_get("passwordhash")?;
    let salt: String = row.try_get("salt")?;
    let token: String = row.try_get("token")?;

    if !verify_password(&algorithm, &password, &username, &salt, &stored_hash) {
        return Err(anyhow::anyhow!("invalid credentials"));
    }

    // Lazy-migrate legacy SHA-256 hashes to Argon2id on successful login.
    // Failures here don't block the login; we'll try again next time.
    if algorithm == PASSWORD_ALGO_SHA256_LEGACY {
        match hash_password(&password) {
            Ok(new) => {
                if let Err(e) = con
                    .execute(
                        "update account set password_algorithm = $1, passwordhash = $2, salt = $3 where id = $4",
                        &[&new.algorithm, &new.hash, &new.salt, &user_id],
                    )
                    .await
                {
                    warn!("failed to migrate password hash for user {user_id}: {e}");
                }
            }
            Err(e) => warn!("failed to compute argon2 hash for migration: {e}"),
        }
    }

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

    let new = hash_password(&password)?;
    let token = uuid::Uuid::new_v4().as_simple().to_string();

    let rows_updated = con
        .execute(
            "insert into account (username, password_algorithm, passwordhash, salt, token) values ($1, $2, $3, $4, $5)",
            &[&username, &new.algorithm, &new.hash, &new.salt, &token],
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
