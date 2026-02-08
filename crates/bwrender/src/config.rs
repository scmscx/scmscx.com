use anyhow::{Context, Result};
use chkdraft_bindings::RenderSkin;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // Database
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_database: String,
    pub db_connections: u32,

    // GSFS
    pub gsfsfe_endpoint: String,

    // Rendering
    pub sc_data_path: String,
    pub render_skin: RenderSkin,
    pub render_batch_size: i64,
    pub render_poll_interval_secs: u64,
    pub render_anim_ticks: u64,
    pub render_webp_quality: f32,

    // Temp directory
    pub temp_dir: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let db_user = env::var("DB_USER").context("DB_USER not set")?;

        Ok(Config {
            db_host: env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            db_port: env::var("DB_PORT")
                .context("DB_PORT not set")?
                .parse()
                .context("DB_PORT must be a number")?,
            db_user: db_user.clone(),
            db_password: env::var("DB_PASSWORD").context("DB_PASSWORD not set")?,
            db_database: env::var("DB_DATABASE").unwrap_or(db_user),
            db_connections: env::var("DB_CONNECTIONS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .context("DB_CONNECTIONS must be a number")?,

            gsfsfe_endpoint: env::var("GSFSFE_ENDPOINT").context("GSFSFE_ENDPOINT not set")?,

            sc_data_path: env::var("SC_DATA_PATH").context("SC_DATA_PATH not set")?,
            render_skin: parse_render_skin(
                &env::var("RENDER_SKIN").unwrap_or_else(|_| "classic".to_string()),
            ),
            render_batch_size: env::var("RENDER_BATCH_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("RENDER_BATCH_SIZE must be a number")?,
            render_poll_interval_secs: env::var("RENDER_POLL_INTERVAL_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("RENDER_POLL_INTERVAL_SECS must be a number")?,
            render_anim_ticks: env::var("RENDER_ANIM_TICKS")
                .unwrap_or_else(|_| "52".to_string())
                .parse()
                .context("RENDER_ANIM_TICKS must be a number")?,
            render_webp_quality: env::var("RENDER_WEBP_QUALITY")
                .unwrap_or_else(|_| "80".to_string())
                .parse()
                .context("RENDER_WEBP_QUALITY must be a number")?,

            temp_dir: env::var("RENDER_TEMP_DIR").unwrap_or_else(|_| "./tmp/render".to_string()),
        })
    }

    pub fn connection_string(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.db_host, self.db_port, self.db_user, self.db_password, self.db_database
        )
    }
}

fn parse_render_skin(s: &str) -> RenderSkin {
    match s.to_lowercase().as_str() {
        "classic" => RenderSkin::Classic,
        "remastered_sd" | "remasteredsd" => RenderSkin::RemasteredSd,
        "remastered_hd2" | "remasteredhd2" => RenderSkin::RemasteredHd2,
        "remastered_hd" | "remasteredhd" => RenderSkin::RemasteredHd,
        "carbot_hd2" | "carbothd2" => RenderSkin::CarbotHd2,
        "carbot_hd" | "carbothd" => RenderSkin::CarbotHd,
        _ => RenderSkin::Classic,
    }
}
