//! Shared application state that used to live in `actix.rs`: the vite manifest
//! chunk type and the cached Backblaze B2 authorization.

use std::sync::Arc;

use anyhow::Result;
use backblaze::api::{b2_authorize_account, B2AuthorizeAccount};
use common::register_counter;
use futures::lock::Mutex;
use serde::Deserialize;

use crate::webutil::Pool;

/// Shared, cheaply-cloneable handles injected into handlers as `Extension`s.
pub type Manifest = Arc<std::collections::HashMap<String, ManifestChunk>>;
pub type Handlebars = Arc<handlebars::Handlebars<'static>>;
pub type BackblazeAuthState = Arc<Mutex<BackblazeAuth>>;

#[derive(Clone, Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ManifestChunk {
    pub file: String,
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub src: String,
    #[allow(dead_code)]
    pub isEntry: Option<bool>,
    pub css: Option<Vec<String>>,
}

#[derive(Default)]
pub struct BackblazeAuth {
    pub version: usize,
    pub auth: Option<B2AuthorizeAccount>,
}

pub async fn get_auth(
    client: &reqwest::Client,
    backblaze_auth: &Mutex<BackblazeAuth>,
    bad_version: Option<usize>,
) -> Result<(usize, B2AuthorizeAccount)> {
    let mut lock = backblaze_auth.lock().await;

    let mut reacquire = false;

    if let Some(bv) = bad_version {
        if lock.version <= bv {
            reacquire = true;
            lock.version = bv;
        }
    }

    if lock.auth.is_none() || reacquire {
        let auth = b2_authorize_account(
            client,
            &std::env::var("BACKBLAZE_KEY_ID").unwrap(),
            &std::env::var("BACKBLAZE_APPLICATION_KEY").unwrap(),
        )
        .await;
        register_counter!(
            "scmscx",
            backblaze_auth,
            "Backblaze B2 authorize-account calls, by result",
            result = if auth.is_ok() { "ok" } else { "error" }
        )
        .inc();
        lock.auth = Some(auth?);

        lock.version = lock.version.checked_add(1).unwrap();
    }

    Ok((lock.version, lock.auth.clone().unwrap()))
}

/// Unused directly here but kept next to the pool type for discoverability.
#[allow(dead_code)]
pub type DbPool = Pool;
