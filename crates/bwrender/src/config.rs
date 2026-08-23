use anyhow::{bail, Context, Result};
use chkdraft_bindings::RenderSkin;
use image::imageops::FilterType;
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

    // Backblaze B2 (fallback). Required unless `backblaze_disabled` is set.
    pub backblaze_disabled: bool,
    pub backblaze_key_id: Option<String>,
    pub backblaze_application_key: Option<String>,

    // Rendering
    pub sc_data_path: String,
    pub render_skin: RenderSkin,
    pub render_poll_interval_secs: u64,
    pub render_anim_ticks: u64,
    pub render_webp_quality: f32,
    /// libwebp encoding effort, 0 (fastest) to 6 (smallest). The default of 4 is
    /// libwebp's own; dropping to 0 is ~4.6x faster at some cost in file size,
    /// which is a good trade for a local dev loop but not for production.
    pub render_webp_method: i32,
    /// Resampling filter for the downscale to `MAX_DIMENSION`. Only applies to
    /// the residual step — sizes that are an exact power-of-two multiple of the
    /// cap are handled by box halving and never reach the resampler.
    pub render_resize_filter: FilterType,
    pub max_concurrent_downloads: usize,
    pub max_concurrent_renders: usize,
    pub max_concurrent_encodes: usize,
    pub max_concurrent_uploads: usize,
    /// Hard cap on how many maps may be anywhere in the pipeline at once,
    /// enforced end to end (download through upload) rather than per stage.
    ///
    /// This is the memory bound. The per-stage limits above cannot provide one:
    /// a raw map image is `width * height * 3` bytes — 768MiB for a 256x256 map
    /// — and rendering is ~12x faster than encoding, so the renderer runs ahead
    /// and fills the encode queue with full-size buffers no matter how few maps
    /// render concurrently. Peak raw pixel memory is roughly this value times the
    /// largest map image, so 1 keeps the renderer inside ~1GiB of raw pixels.
    pub max_maps_in_flight: usize,

    // Temp directory
    pub temp_dir: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let db_user = env::var("DB_USER").context("DB_USER not set")?;

        // Require Backblaze credentials unless Backblaze is explicitly disabled,
        // so a misconfiguration fails loudly instead of silently not working.
        let backblaze_disabled = env::var("BACKBLAZE_DISABLED").as_deref() == Ok("true");
        let (backblaze_key_id, backblaze_application_key) = if backblaze_disabled {
            (None, None)
        } else {
            (
                Some(env::var("BACKBLAZE_KEY_ID").context(
                    "BACKBLAZE_KEY_ID not set (set BACKBLAZE_DISABLED=true to run without Backblaze)",
                )?),
                Some(env::var("BACKBLAZE_APPLICATION_KEY").context(
                    "BACKBLAZE_APPLICATION_KEY not set (set BACKBLAZE_DISABLED=true to run without Backblaze)",
                )?),
            )
        };

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

            backblaze_disabled,
            backblaze_key_id,
            backblaze_application_key,

            sc_data_path: env::var("SC_DATA_PATH").context("SC_DATA_PATH not set")?,
            render_skin: parse_render_skin(
                &env::var("RENDER_SKIN").unwrap_or_else(|_| "classic".to_string()),
            ),
            render_poll_interval_secs: env::var("RENDER_POLL_INTERVAL_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("RENDER_POLL_INTERVAL_SECS must be a number")?,
            render_anim_ticks: env::var("RENDER_ANIM_TICKS")
                .unwrap_or_else(|_| "52".to_string())
                .parse()
                .context("RENDER_ANIM_TICKS must be a number")?,
            render_webp_quality: env::var("RENDER_WEBP_QUALITY")
                .unwrap_or_else(|_| "90".to_string())
                .parse()
                .context("RENDER_WEBP_QUALITY must be a number")?,
            render_webp_method: parse_webp_method(
                &env::var("RENDER_WEBP_METHOD").unwrap_or_else(|_| "4".to_string()),
            )?,
            render_resize_filter: parse_resize_filter(
                &env::var("RENDER_RESIZE_FILTER").unwrap_or_else(|_| "lanczos3".to_string()),
            )?,
            max_concurrent_downloads: env::var("RENDER_MAX_CONCURRENT_DOWNLOADS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("RENDER_MAX_CONCURRENT_DOWNLOADS must be a number")?,
            max_concurrent_renders: env::var("RENDER_MAX_CONCURRENT_RENDERS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .context("RENDER_MAX_CONCURRENT_RENDERS must be a number")?,
            max_concurrent_encodes: env::var("RENDER_MAX_CONCURRENT_ENCODES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("RENDER_MAX_CONCURRENT_ENCODES must be a number")?,
            max_concurrent_uploads: env::var("RENDER_MAX_CONCURRENT_UPLOADS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("RENDER_MAX_CONCURRENT_UPLOADS must be a number")?,
            max_maps_in_flight: env::var("RENDER_MAX_MAPS_IN_FLIGHT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("RENDER_MAX_MAPS_IN_FLIGHT must be a number")
                .and_then(|n: usize| {
                    // A zero here would wedge the pipeline: every map would block
                    // forever waiting for a permit that can never be issued.
                    if n == 0 {
                        anyhow::bail!("RENDER_MAX_MAPS_IN_FLIGHT must be at least 1");
                    }
                    Ok(n)
                })?,

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

/// Parse `RENDER_WEBP_METHOD`. Unlike [`parse_render_skin`] this rejects bad
/// input rather than falling back: silently encoding at the default when you
/// asked for method 0 would quietly invalidate whatever you were measuring.
fn parse_webp_method(s: &str) -> Result<i32> {
    let method: i32 = s
        .parse()
        .context("RENDER_WEBP_METHOD must be a number from 0 to 6")?;
    if !(0..=6).contains(&method) {
        bail!("RENDER_WEBP_METHOD must be from 0 (fastest) to 6 (smallest), got {method}");
    }
    Ok(method)
}

/// Parse `RENDER_RESIZE_FILTER`. Rejects unknown names for the same reason as
/// [`parse_webp_method`].
fn parse_resize_filter(s: &str) -> Result<FilterType> {
    Ok(match s.to_lowercase().as_str() {
        "nearest" => FilterType::Nearest,
        "triangle" => FilterType::Triangle,
        "catmullrom" | "catmull_rom" => FilterType::CatmullRom,
        "gaussian" => FilterType::Gaussian,
        "lanczos3" => FilterType::Lanczos3,
        other => bail!(
            "RENDER_RESIZE_FILTER must be one of nearest, triangle, catmullrom, gaussian, lanczos3; got {other:?}"
        ),
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webp_method_accepts_the_valid_range() {
        for m in 0..=6 {
            assert_eq!(parse_webp_method(&m.to_string()).unwrap(), m);
        }
    }

    #[test]
    fn webp_method_rejects_out_of_range_and_garbage() {
        // Silently clamping would make a benchmark run at a method you didn't ask
        // for, so these must be hard errors.
        for bad in ["-1", "7", "100", "fast", ""] {
            assert!(
                parse_webp_method(bad).is_err(),
                "{bad:?} should be rejected"
            );
        }
    }

    #[test]
    fn resize_filter_parses_known_names_case_insensitively() {
        assert!(matches!(
            parse_resize_filter("Lanczos3").unwrap(),
            FilterType::Lanczos3
        ));
        assert!(matches!(
            parse_resize_filter("triangle").unwrap(),
            FilterType::Triangle
        ));
        assert!(matches!(
            parse_resize_filter("CATMULLROM").unwrap(),
            FilterType::CatmullRom
        ));
        assert!(matches!(
            parse_resize_filter("catmull_rom").unwrap(),
            FilterType::CatmullRom
        ));
        assert!(matches!(
            parse_resize_filter("nearest").unwrap(),
            FilterType::Nearest
        ));
    }

    #[test]
    fn resize_filter_rejects_unknown_names() {
        let err = parse_resize_filter("bilinear").unwrap_err().to_string();
        assert!(
            err.contains("bilinear"),
            "error should name the input: {err}"
        );
        assert!(
            err.contains("triangle"),
            "error should list the valid options: {err}"
        );
    }
}
