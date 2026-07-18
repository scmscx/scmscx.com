//! End-to-end tests for the map lifecycle: upload, view, the per-map metadata
//! endpoints, flags, tags, NSFW access control, and search.
//!
//! These need a real StarCraft map, because the upload endpoint parses the MPQ
//! and rejects anything that isn't one. We don't ship a fixture; instead each run
//! fetches one specific, hash-pinned map (see [`MAP_SHA256`]) from the origin via
//! `GET /api/maps/{hash}` and re-uploads it into the isolated per-test database.
//! Pinning the exact map (rather than picking a random featured one) keeps runs
//! deterministic. We hit the origin over its wireguard IP directly (like the
//! `bwmap` test util does) rather than `scmscx.com`, to bypass the reverse proxy's
//! rate limiting. The download is memoized for the whole test-binary run (see
//! [`sample_map`]), so the network is hit once, not per test.
//!
//! Note we can't round-trip the *download* (`GET /api/maps/{hash}`): that streams
//! from gsfs/Backblaze, which are off in the harness (upload only stages the blob
//! to a local `./pending/` dir). So "viewing" here means the metadata endpoints,
//! not fetching the map bytes back.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tokio::sync::OnceCell;

use crate::harness::{client, cookie_value, json_body, Harness};

/// The origin server we borrow a real map from — reached over its wireguard IP
/// directly (as `bwmap`'s test util does) to skip the reverse proxy's rate limit.
const UPSTREAM: &str = "http://10.99.99.5:5000";

// ---------------------------------------------------------------------------
// Fixture: one specific, hash-pinned map, fetched from the origin at most once
// per machine and then served from an on-disk cache.
// ---------------------------------------------------------------------------

/// sha256 of the map we upload — also its mapblob hash, so `/api/maps/{this}`
/// serves exactly these bytes. It's the scmscx web-id `5sBzzTHk` ("LotR: The March
/// of Sauron II V4.3"), a *featured* map, so it won't be deleted out from under us.
/// The upload endpoint re-hashes the body and checks it against this, so a wrong or
/// corrupt download fails the upload loudly.
const MAP_SHA256: &str = "ef2390f7758f1ab573f74aed8124795b65169d0a4265ced7624428a1e088dbff";
/// Expected byte length of that map — a cheap sanity check on the download.
const MAP_LEN: usize = 2_123_577;

// Extra hash-pinned fixtures used where the default UMS map can't exercise a code
// path. All are featured/permanent maps fetched and hash-checked exactly like the
// default one; each `const` pairs the mapblob sha256 with its byte length.

/// A small melee (ladder) map — "| iCCup | Fighting Spirit 1.3" (web id `qsH9qbTG`).
/// Its forces carry the *opposite* property bits to the LotR UMS map — a Melee force
/// with random start locations and no allies/allied-victory/shared-vision, next to
/// an all-shared Observer force — which exercises the force-property bit decoding in
/// the states the single UMS map leaves constant.
const FIGHTING_SPIRIT_SHA256: &str =
    "15c7ca64d2e57088fe97c00c700e20eb0a65728ebce93e0373a85c8ec035f041";
const FIGHTING_SPIRIT_LEN: usize = 70_819;

/// An EUD map — "Income Wars (with EUDs)" (web id `Vch33HpH`). Its triggers reference
/// extended unit-death addresses outside the normal death table, so its
/// get/set-death-EUD counters are non-zero — unlike a normal map, where they're all
/// zero and the counting logic can't be observed.
const INCOME_WARS_SHA256: &str = "3b77b0f17e3e757a776882379bc9c7c2f55c5eac2e037f08a48040d5191ba8af";
const INCOME_WARS_LEN: usize = 487_789;

/// An extended-unit (EUP) map with placed units whose *owner* is out of the normal
/// 0..=11 range — "포커 디펜스 1.2v" (web id `7pfzG5bc`). `get_eups` (which selects
/// units with `owner > 12 || unit_id > 227`) returns 39 units here (all owner-side),
/// exercising the `owner > 12` comparison a normal map leaves empty.
const POKER_DEFENSE_SHA256: &str =
    "d7b56e2a9004ed2d8feebae185b77ee76889f9a86beeb0ed232869302e668aad";
const POKER_DEFENSE_LEN: usize = 99_420;

/// An EUP map with placed units whose *unit id* is out of the normal 0..=227 range —
/// "Untitled Scenario" (web id `cgD56Bkk`). `get_eups` returns 7 units here (all
/// unit-id-side, owners in range), exercising the `unit_id > 227` comparison.
const UNTITLED_EUP_SHA256: &str =
    "dad825509d47223bd11c21eaf993cc873f3a78dc4e2ae61c2294e78fb7d18701";
const UNTITLED_EUP_LEN: usize = 545_106;

/// On-disk cache location for a pinned map, shared across test *processes*. Keyed by
/// the map's sha256, so bumping a fixture hash automatically misses the old entry.
/// Lives under the system temp dir (swept on reboot).
fn map_cache_path(sha256: &str) -> PathBuf {
    env::temp_dir().join("scmscx-e2e-map-cache").join(sha256)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

/// Fetch a hash-pinned map's bytes: served from the on-disk cache when a previous
/// process left an intact copy, otherwise fetched once from the origin and cached.
/// Shared by every fixture (keyed by sha256).
///
/// Why the disk cache: cargo-mutants reruns the whole suite in a fresh process per
/// mutant, so without it each fixture would be re-downloaded hundreds of times per
/// `make mutants` sweep, hammering the origin (which we reach over wireguard
/// specifically to bypass the reverse proxy's rate limiter, so nothing would
/// throttle it). One cached copy turns that back into a single download per map.
async fn download_and_cache(sha256: &str, len: usize) -> Vec<u8> {
    let cache = map_cache_path(sha256);

    // Fast path: a cached copy that still matches the pinned hash. A partial or
    // corrupt file fails the checks and falls through to a fresh fetch.
    if let Ok(bytes) = fs::read(&cache) {
        if bytes.len() == len && sha256_hex(&bytes) == sha256 {
            return bytes;
        }
    }

    let c = Client::builder()
        .timeout(Duration::from_mins(1))
        .build()
        .unwrap();

    // "norecord" in the UA suppresses the origin's download counter.
    let bytes = c
        .get(format!("{UPSTREAM}/api/maps/{sha256}"))
        .header("user-agent", "scmscx-e2e-norecord")
        .send()
        .await
        .expect("GET map blob from the origin")
        .error_for_status()
        .expect("map blob has a success status")
        .bytes()
        .await
        .expect("read map blob body")
        .to_vec();
    assert_eq!(
        bytes.len(),
        len,
        "pinned map {sha256} changed size unexpectedly"
    );
    assert_eq!(
        sha256_hex(&bytes),
        sha256,
        "origin served a map whose sha256 doesn't match {sha256}"
    );

    // Cache for the next process: write to a unique temp file then rename, so a
    // concurrent reader never observes a partial file and racing writers just both
    // land identical bytes.
    if let Some(dir) = cache.parent() {
        if fs::create_dir_all(dir).is_ok() {
            let tmp = dir.join(format!(".{}.{}.tmp", sha256, std::process::id()));
            if fs::write(&tmp, &bytes).is_ok() {
                let _ = fs::rename(&tmp, &cache);
            }
        }
    }

    bytes
}

/// The default fixture (LotR: The March of Sauron II) — a large UMS map used by most
/// tests via [`upload_map`]. Memoized so one test binary reads it at most once.
async fn sample_map() -> &'static [u8] {
    static CELL: OnceCell<Vec<u8>> = OnceCell::const_new();
    CELL.get_or_init(|| download_and_cache(MAP_SHA256, MAP_LEN))
        .await
}

// ---------------------------------------------------------------------------
// Auth + upload helpers.
// ---------------------------------------------------------------------------

/// A logged-in account, carried as a cookie header on subsequent requests.
struct Auth {
    username: String,
    token: String,
}

impl Auth {
    fn cookie(&self) -> String {
        format!("username={}; token={}", self.username, self.token)
    }
}

/// Register `username` and return its session cookies.
async fn register(c: &Client, h: &Harness, username: &str) -> Auth {
    let resp = c
        .post(h.url("/api/register"))
        .header("content-type", "application/json")
        .body(
            serde_json::json!({
                "username": username,
                "password": "pw",
                "password_confirm": "pw",
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "register {username}");
    let token = cookie_value(&resp, "token").expect("register sets a token cookie");
    Auth {
        username: username.to_string(),
        token,
    }
}

/// Upload a map body into this harness under `filename`, declaring `sha256` (the
/// upload endpoint re-hashes the body and checks it). When `auth` is set the map is
/// owned by that account (required to later flag/tag it). Returns the new map's web
/// id — the same identifier search results and URLs use.
async fn upload_bytes(
    c: &Client,
    h: &Harness,
    auth: Option<&Auth>,
    filename: &str,
    sha256: &str,
    bytes: Vec<u8>,
) -> String {
    // Metadata rides in the query string; the raw bytes are the request body.
    let url = h.url(&format!(
        "/api/uiv2/upload-map?filename={filename}&sha256={sha256}&last_modified=1700000000000&length={}&playlist=e2e",
        bytes.len(),
    ));
    let mut req = c.post(url).body(bytes);
    if let Some(a) = auth {
        req = req.header("cookie", a.cookie());
    }
    let resp = req.send().await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "upload-map should return 200"
    );
    let body = json_body(resp).await;
    // Success is a JSON string (the web id); a rejected/blacklisted map is `-1`.
    body.as_str()
        .unwrap_or_else(|| panic!("upload returned a non-string body (map rejected?): {body}"))
        .to_string()
}

/// Upload the default sample map ([`sample_map`]). Most tests use this.
async fn upload_map(c: &Client, h: &Harness, auth: Option<&Auth>, filename: &str) -> String {
    upload_bytes(
        c,
        h,
        auth,
        filename,
        MAP_SHA256,
        sample_map().await.to_vec(),
    )
    .await
}

/// Upload an arbitrary hash-pinned fixture (e.g. [`FIGHTING_SPIRIT_SHA256`]).
async fn upload_fixture(
    c: &Client,
    h: &Harness,
    auth: Option<&Auth>,
    filename: &str,
    sha256: &str,
    len: usize,
) -> String {
    let bytes = download_and_cache(sha256, len).await;
    upload_bytes(c, h, auth, filename, sha256, bytes).await
}

/// The web ids returned by `GET /api/uiv2/search/{query}`.
async fn search_ids(c: &Client, h: &Harness, query: &str) -> Vec<String> {
    let resp = c
        .get(h.url(&format!("/api/uiv2/search/{query}")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "search {query}");
    json_body(resp).await["maps"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["id"].as_str().map(str::to_string))
        .collect()
}

/// The numeric internal (DB) id of a map, read back from `map_info`. Needed to
/// fetch the map's `chkblob` hash via the harness DB helper for the chk-hash-keyed
/// endpoints.
async fn map_internal_id(c: &Client, h: &Harness, web_id: &str) -> i64 {
    let info = json_body(
        c.get(h.url(&format!("/api/uiv2/map_info/{web_id}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    info["internal_id"]
        .as_i64()
        .expect("map_info carries a numeric internal_id")
}

/// Build a `[{key,value},...]` tags request body.
fn tags_body(pairs: &[(&str, &str)]) -> String {
    serde_json::Value::Array(
        pairs
            .iter()
            .map(|(k, v)| serde_json::json!({ "key": k, "value": v }))
            .collect(),
    )
    .to_string()
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

/// The server-rendered `/map/{id}` page. Three things are pinned: (1) a numeric id
/// 301-redirects to its web-id URL while a non-numeric or too-long id does not
/// (guarding the `all_numeric && len < 8` routing predicate and its boundary);
/// (2) the parsed scenario name and description are rendered into the HTML (guarding
/// the string-number lookups); (3) a normal map's page is served (200) to an
/// anonymous visitor (guarding the blackhole gate against an `&&`→`||` flip that
/// would 404 every map).
#[tokio::test]
async fn map_page_ssr_renders_and_routes() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "ssrmapowner").await;
    let id = upload_map(&c, &h, Some(&owner), "e2essrmap.scx").await;

    // A short numeric id is a raw DB id → 301 to its web-id URL.
    assert_eq!(
        c.get(h.url("/map/123")).send().await.unwrap().status(),
        StatusCode::PERMANENT_REDIRECT,
        "a numeric map id redirects to its web id"
    );
    // Non-numeric (decoded as a web id, which fails) and 8+ all-digit (past the
    // `len < 8` guard, so treated as a web id, which fails) are both 404 — not
    // redirects.
    for bad in ["/map/abc", "/map/12345678"] {
        assert_eq!(
            c.get(h.url(bad)).send().await.unwrap().status(),
            StatusCode::NOT_FOUND,
            "{bad} is a 404, not a redirect"
        );
    }

    // The real map's page renders (200, anonymously) with its scenario name and
    // description baked into the HTML server-side.
    let resp = c.get(h.url(&format!("/map/{id}"))).send().await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "a normal map's page is visible to anonymous visitors"
    );
    let html = resp.text().await.unwrap();
    assert!(
        html.contains("March of Sauron"),
        "the scenario name is rendered into the page"
    );
    assert!(
        html.contains("War of the Ring"),
        "the scenario description is rendered into the page"
    );
}

/// The image endpoints' access control. `get_map_image` (`/api/uiv2/img`) has no
/// local fallback with GSFS disabled, so a viewable map is a 404 — which means an
/// SFW map served anonymously must be a 404, *not* the 403 an over-eager NSFW gate
/// (`nsfw && anon` flipped to `||`) would produce. The minimap endpoint does have a
/// local fallback, so its blackhole gate is observable: a blackholed map's minimap
/// is 404 to an anonymous viewer but still served to its owner.
#[tokio::test]
async fn image_endpoints_gate_access() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "imgowner").await;
    let id = upload_map(&c, &h, Some(&owner), "e2eimg.scx").await;

    // SFW map, anonymous: the image is a 404 (no GSFS), never a 403.
    assert_eq!(
        c.get(h.url(&format!("/api/uiv2/img/{id}")))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND,
        "an SFW map's image is 404 (not 403) for an anonymous viewer"
    );

    // Blackhole it, then check the minimap's owner-vs-anonymous gate.
    assert_eq!(
        c.post(h.url(&format!("/api/flags/{id}/blackholed")))
            .header("cookie", owner.cookie())
            .header("content-type", "application/json")
            .body("true")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        c.get(h.url(&format!("/api/uiv2/minimap/{id}")))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND,
        "a blackholed map's minimap is hidden from anonymous viewers"
    );
    assert_eq!(
        c.get(h.url(&format!("/api/uiv2/minimap/{id}")))
            .header("cookie", owner.cookie())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "the owner can still fetch a blackholed map's minimap"
    );
}

/// `map_info` doesn't just echo DB columns — it parses the CHK and derives a whole
/// `properties`/`forces`/`wavs` structure. This pins that derivation against the
/// hash-fixed fixture map so a mutation to any of the parsing/bit-twiddling/counting
/// logic (force-property flag bits, the location-degeneracy filter, the wav-action
/// match arms, the scenario-description lookup) changes an asserted value and fails.
///
/// The fixture is a normal (non-EUD) UMS map, so its EUP / death-EUD / trigger-list
/// counters are all zero; the mutants over *those* branches (map_info.rs ~250–306)
/// can only be killed by a map that actually contains extended-unit-death triggers,
/// which this fixture doesn't — see the suite notes.
#[tokio::test]
async fn map_info_reports_parsed_chk_structure() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "structowner").await;
    let id = upload_map(&c, &h, Some(&owner), "e2estruct.scx").await;

    let info = json_body(
        c.get(h.url(&format!("/api/uiv2/map_info/{id}")))
            .send()
            .await
            .unwrap(),
    )
    .await;

    // Scenario name + description both come from string-table lookups gated on a
    // non-zero string number; the description lookup in particular is otherwise
    // unpinned.
    assert_eq!(info["scenario"], "LotR: The March of Sauron II V4.3");
    assert_eq!(
        info["scenario_description"],
        "Activate the nations of Middle Earth to fight The War of the Ring!"
    );

    // Per-player owner/side arrays are echoed straight from OWNR/SIDE.
    assert_eq!(
        info["player_owners"],
        serde_json::json!([6, 6, 6, 6, 6, 6, 6, 6, 0, 0, 0, 0])
    );
    assert_eq!(
        info["player_side"],
        serde_json::json!([1, 1, 1, 1, 1, 1, 1, 1, 7, 7, 7, 7])
    );

    // Forces: names (string-table vs the "Force N" fallback for string number 0),
    // the four property flags decoded bit-by-bit from force_properties, and the
    // players grouped into each force by equality on the force id.
    let force = |name: &str, players: serde_json::Value| {
        serde_json::json!({
            "name": name,
            "player_ids": players,
            "prop_random_start_location": false,
            "prop_allies": true,
            "prop_allied_victory": true,
            "prop_shared_vision": true,
        })
    };
    assert_eq!(
        info["forces"],
        serde_json::json!([
            force(
                "Gondor-Rohan-Eriador-Rhovanion",
                serde_json::json!([0, 1, 2, 3])
            ),
            force(
                "Mordor-Isen-Misty Mnts-Dol Guldur",
                serde_json::json!([4, 5, 6, 7])
            ),
            force("Force 3", serde_json::json!([])),
            force("Force 4", serde_json::json!([])),
        ]),
        "forces decode names, per-force players, and the four property bits"
    );

    // Derived counts over the parsed sections. `locations` in particular is a
    // filtered count (degenerate zero-area locations are dropped), which pins the
    // whole `!(left==right || top==bottom)` predicate.
    let props = &info["properties"];
    assert_eq!(props["ver"], 206);
    assert_eq!(props["width"], 256);
    assert_eq!(props["height"], 256);
    assert_eq!(props["tileset"], 0);
    assert_eq!(props["doodads"], 742);
    assert_eq!(props["sprites"], 416);
    assert_eq!(props["triggers"], 817);
    assert_eq!(props["briefing_triggers"], 8);
    assert_eq!(props["units"], 930);
    assert_eq!(props["unique_terrain_tiles"], 2147);
    assert_eq!(
        props["locations"], 219,
        "degenerate locations are filtered out"
    );

    // A normal map references only the in-range death table, so every EUD/EUP
    // counter is zero. Pinning them still kills any mutation that would make a
    // boundary unit/offset tip a counter positive on this map.
    assert_eq!(props["eups"], 0);
    assert_eq!(props["get_death_euds"], 0);
    assert_eq!(props["set_death_euds"], 0);
    assert_eq!(props["trigger_list_reads"], 0);
    assert_eq!(props["trigger_list_writes"], 0);

    // `wavs` is the de-duplicated set of every wave referenced by a PlayWav or
    // Transmission action (trigger or mission-briefing). Pinning the exact set kills
    // deletion of any wav-contributing match arm. HashSet order isn't stable, so
    // compare sorted.
    let mut wavs: Vec<String> = info["wavs"]
        .as_array()
        .expect("wavs is an array")
        .iter()
        .map(|w| w.as_str().unwrap().to_string())
        .collect();
    wavs.sort();
    let mut expected = vec![
        r"staredit\wav\A_Chantar.ogg",
        r"staredit\wav\City-Capture.ogg",
        r"staredit\wav\Dawrf.ogg",
        r"staredit\wav\Eriador.ogg",
        r"staredit\wav\Gondor.ogg",
        r"staredit\wav\Isen.ogg",
        r"staredit\wav\Khun March.ogg",
        r"staredit\wav\Mordor.ogg",
        r"staredit\wav\Moria.ogg",
        r"staredit\wav\Rohan.ogg",
        r"staredit\wav\VOXScrm_Wilhelm scream (ID 0477)_BSB.wav",
        r"staredit\wav\overrun.ogg",
        r"staredit\wav\sauron_welcome.wav",
    ];
    expected.sort_unstable();
    assert_eq!(wavs, expected, "the exact set of referenced wavs");
}

/// The four force-property flags are decoded bit-by-bit from `force_properties`. The
/// default UMS fixture leaves every force with the same bits set (random-start off,
/// the other three on), so a mutation to a bit that's constant across its forces is
/// invisible. This melee map carries the opposite polarity — a Melee force with
/// random-start ON and allies/allied-victory/shared-vision OFF, beside an all-on
/// Observer force — so each flag is asserted in both states and every bit's decode
/// (`&` mask, shift, and `> 0` test) is pinned.
#[tokio::test]
async fn map_info_force_properties_from_melee_map() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "meleeowner").await;
    let id = upload_fixture(
        &c,
        &h,
        Some(&owner),
        "e2emelee.scx",
        FIGHTING_SPIRIT_SHA256,
        FIGHTING_SPIRIT_LEN,
    )
    .await;

    let info = json_body(
        c.get(h.url(&format!("/api/uiv2/map_info/{id}")))
            .send()
            .await
            .unwrap(),
    )
    .await;

    let force = |name: &str, players: serde_json::Value, rand, ally, avic, vis| {
        serde_json::json!({
            "name": name,
            "player_ids": players,
            "prop_random_start_location": rand,
            "prop_allies": ally,
            "prop_allied_victory": avic,
            "prop_shared_vision": vis,
        })
    };
    assert_eq!(
        info["forces"],
        serde_json::json!([
            // Melee force: random-start ON, everything else OFF.
            force(
                "Players",
                serde_json::json!([0, 1, 2, 3]),
                true,
                false,
                false,
                false
            ),
            // Observer force: random-start OFF, everything else ON.
            force(
                "Observers",
                serde_json::json!([4, 5, 6, 7]),
                false,
                true,
                true,
                true
            ),
            force("Force 3", serde_json::json!([]), true, true, true, true),
            force("Force 4", serde_json::json!([]), true, true, true, true),
        ]),
        "force-property bits decode correctly in both polarities"
    );
}

/// `map_info` counts extended-unit-death (EUD) trigger references — reads and writes
/// of unit-death addresses that fall *outside* the normal death table. A normal map
/// has none (all these counters are zero), so the range checks and increments can't
/// be observed there. This EUD map does, so its `get_death_euds`/`set_death_euds`
/// counters are non-zero and pin that logic.
#[tokio::test]
async fn map_info_counts_extended_unit_deaths() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "eudowner").await;
    let id = upload_fixture(
        &c,
        &h,
        Some(&owner),
        "e2eeud.scx",
        INCOME_WARS_SHA256,
        INCOME_WARS_LEN,
    )
    .await;

    let props = json_body(
        c.get(h.url(&format!("/api/uiv2/map_info/{id}")))
            .send()
            .await
            .unwrap(),
    )
    .await["properties"]
        .clone();
    assert_eq!(
        props["get_death_euds"], 60,
        "EUD death-read references are counted"
    );
    assert_eq!(
        props["set_death_euds"], 2020,
        "EUD death-write references are counted"
    );

    // The separate `trigger_list_reads`/`trigger_list_writes` counters tally EUD
    // references landing in the trigger-list region [0x51A280, 0x51A2E0). This map's
    // 60 death-reads and 2020 death-writes all sit elsewhere, so both are 0 — and
    // pinning that here (on a map that HAS heavy EUD traffic) kills the mutants that
    // widen the range check to sweep those references in: an `&&`→`||` makes it match
    // everything, and shifting a bound (`>=`→`<`, `<`→`>`) pulls this map's offsets
    // into range. The remaining boundary flips (`==`/`<=`) and the `+=` increments
    // need an offset that actually lands in the 96-byte window, which no available
    // fixture has — those are excluded in .cargo/mutants.toml.
    assert_eq!(
        props["trigger_list_reads"], 0,
        "none of this map's death-reads reference the trigger-list region"
    );
    assert_eq!(
        props["trigger_list_writes"], 0,
        "none of this map's death-writes reference the trigger-list region"
    );
}

/// `map_info`'s `eups` property counts placed units that are "extended" — unit id
/// out of 0..=227 (`unit_id > 227`) OR owner out of 0..=27 (`owner > 27`). A normal
/// map has none (pinned to 0 in `map_info_reports_parsed_chk_structure`), so the two
/// comparisons, their `||`, and the `+=` go untested there. These EUP fixtures each
/// populate exactly one side of the `||`: the counts (39 owner-side, 7 unit-id-side)
/// pin the whole expression — `||`→`&&` collapses either fixture's count to 0, a
/// `>`→`==` drops that fixture's whole set, and `+=`→`-=`/`*=` breaks the tally. (The
/// `>`→`>=` boundary — a placed unit of type exactly 227 or owner exactly 27 — is not
/// reachable with any available fixture and is excluded in .cargo/mutants.toml.)
///
/// Note this counter differs from `GET /api/chk/eups` (chk.rs `get_eups`), which uses
/// `owner > 12`; here the owner threshold is 27, so the two are pinned separately.
#[tokio::test]
async fn map_info_counts_extended_unit_placements() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "eupplacementowner").await;

    // Owner-extended (owner > 27, unit_id <= 227): 39 placements.
    let poker = upload_fixture(
        &c,
        &h,
        Some(&owner),
        "poker_placements.scx",
        POKER_DEFENSE_SHA256,
        POKER_DEFENSE_LEN,
    )
    .await;
    let poker_props = json_body(
        c.get(h.url(&format!("/api/uiv2/map_info/{poker}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        poker_props["properties"]["eups"], 39,
        "the owner-extended EUP map has 39 extended placements"
    );

    // Unit-id-extended (unit_id > 227, owner in range): 7 placements.
    let untitled = upload_fixture(
        &c,
        &h,
        Some(&owner),
        "untitled_placements.scx",
        UNTITLED_EUP_SHA256,
        UNTITLED_EUP_LEN,
    )
    .await;
    let untitled_props = json_body(
        c.get(h.url(&format!("/api/uiv2/map_info/{untitled}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        untitled_props["properties"]["eups"], 7,
        "the unit-id-extended EUP map has 7 extended placements"
    );
}

/// Upload a map as a logged-in user, then read it back through the view/metadata
/// endpoints: details name us as the uploader, and the filename we chose shows up.
#[tokio::test]
async fn map_upload_and_view() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "mapowner").await;
    let filename = "e2eviewmap.scx";
    let id = upload_map(&c, &h, Some(&owner), filename).await;

    // map_info: 200, a scenario name, and we are recorded as the uploader.
    let resp = c
        .get(h.url(&format!("/api/uiv2/map_info/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let info = json_body(resp).await;
    assert!(
        info["scenario"].is_string(),
        "scenario name present: {info}"
    );
    assert!(info["internal_id"].is_number());
    assert_eq!(
        info["meta"]["uploaded_by"].as_str(),
        Some("mapowner"),
        "the logged-in uploader owns the map"
    );

    // filenames / filenames2 include the name we uploaded under.
    let names = json_body(
        c.get(h.url(&format!("/api/uiv2/filenames/{id}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert!(
        names
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str() == Some(filename)),
        "filenames should include {filename}: {names}"
    );

    let names2 = json_body(
        c.get(h.url(&format!("/api/uiv2/filenames2/{id}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert!(
        names2
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["filename"].as_str() == Some(filename)),
        "filenames2 should include {filename}: {names2}"
    );

    // timestamps and units are JSON arrays; replays is empty on a fresh map.
    for path in ["timestamps", "units"] {
        let resp = c
            .get(h.url(&format!("/api/uiv2/{path}/{id}")))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "GET {path}");
        assert!(
            json_body(resp).await.is_array(),
            "{path} should be an array"
        );
    }
    let replays = json_body(
        c.get(h.url(&format!("/api/uiv2/replays/{id}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(replays, serde_json::json!([]), "a fresh map has no replays");
}

/// After a successful upload the raw map is staged under `./pending/` — one blob
/// per delivery backend, named by hash — for the out-of-process pumpers to ship to
/// gsfs/Backblaze. Assert both land in the right place, correctly named, with the
/// exact bytes we uploaded.
#[tokio::test]
async fn upload_stages_blob_by_hash_for_delivery() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "stageowner").await;
    upload_map(&c, &h, Some(&owner), "e2estage.scx").await;

    let bytes = sample_map().await;
    let pending = h.pending_dir();

    // gsfs: a file named exactly by the sha256, byte-identical to the map.
    let gsfs_blob = pending.join("gsfs").join(MAP_SHA256);
    let staged = std::fs::read(&gsfs_blob)
        .unwrap_or_else(|e| panic!("gsfs blob {gsfs_blob:?} should exist: {e}"));
    assert_eq!(
        staged.as_slice(),
        bytes,
        "gsfs blob must be the uploaded map's bytes"
    );

    // backblaze: a single file named "{sha1}-{sha256}".
    let bb_dir = pending.join("backblaze");
    let bb: Vec<String> = std::fs::read_dir(&bb_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(bb.len(), 1, "exactly one backblaze blob, got {bb:?}");
    let (sha1, sha256) = bb[0]
        .split_once('-')
        .expect("backblaze blob is named {sha1}-{sha256}");
    assert_eq!(
        sha256, MAP_SHA256,
        "backblaze blob's sha256 suffix matches the map"
    );
    assert_eq!(
        sha1.len(),
        40,
        "backblaze blob's prefix is a sha1 hex digest"
    );
    assert!(
        sha1.chars().all(|ch| ch.is_ascii_hexdigit()),
        "sha1 prefix is hex: {sha1}"
    );
    assert_eq!(
        std::fs::read(bb_dir.join(&bb[0])).unwrap().as_slice(),
        bytes,
        "backblaze blob must be the uploaded map's bytes"
    );

    // The scratch copy used during staging is cleaned up.
    assert_eq!(
        std::fs::read_dir(pending.join("tmp")).unwrap().count(),
        0,
        "pending/tmp should be empty after a successful upload"
    );
}

/// The units endpoint lists exactly the map's custom-named units — those that are
/// enabled (`config == 0`) *and* carry a name string (`string_number != 0`),
/// ordered by unit id. Pinning the count guards that two-part filter: dropping
/// either half, flipping the equality, or turning the `&&` into `||` changes which
/// units qualify and so changes the length.
#[tokio::test]
async fn units_endpoint_lists_named_units() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "unitsowner").await;
    let id = upload_map(&c, &h, Some(&owner), "e2eunits.scx").await;

    let units = json_body(
        c.get(h.url(&format!("/api/uiv2/units/{id}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    let arr = units.as_array().expect("units is an array");
    assert_eq!(
        arr.len(),
        124,
        "the fixture map has exactly 124 custom-named units"
    );
    // Ordered by unit id ascending, each entry carries a non-empty name.
    let ids: Vec<i64> = arr.iter().map(|u| u["unit_id"].as_i64().unwrap()).collect();
    assert_eq!(ids[0], 0, "first custom unit is unit id 0");
    assert!(ids.windows(2).all(|w| w[0] < w[1]), "unit ids ascend");
    assert!(
        arr.iter()
            .all(|u| u["name"].as_str().is_some_and(|n| !n.is_empty())),
        "every listed unit has a name"
    );
}

/// The paginated map sitemap `/a.txt` lists the web ids of visible maps (first 50k,
/// offset 0). On a fresh DB it's empty; after an upload it must contain that map's
/// URL. The empty-DB case alone can't tell the real handler from one that always
/// returns nothing, so this uploads a map and asserts it appears.
#[tokio::test]
async fn sitemap_a_txt_lists_uploaded_map() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "sitemapowner").await;
    let id = upload_map(&c, &h, Some(&owner), "e2esitemap.scx").await;

    let body = c
        .get(h.url("/a.txt"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(
        body,
        format!("https://scmscx.com/map/{id}\n"),
        "a.txt lists the uploaded map's URL"
    );
}

/// Flags: readable by anyone, but writing needs login *and* ownership. Unknown
/// flag names are 404. A set value round-trips through get.
#[tokio::test]
async fn map_flags_require_ownership() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "flagowner").await;
    let intruder = register(&c, &h, "flagintruder").await;
    let id = upload_map(&c, &h, Some(&owner), "e2eflags.scx").await;

    let flag_url = |flag: &str| h.url(&format!("/api/flags/{id}/{flag}"));
    let set = |flag: &str, val: &str, cookie: Option<&str>| {
        let mut req = c
            .post(flag_url(flag))
            .header("content-type", "application/json")
            .body(val.to_string());
        if let Some(ck) = cookie {
            req = req.header("cookie", ck.to_string());
        }
        req.send()
    };

    // Default value is false, readable without auth.
    let resp = c.get(flag_url("nsfw")).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        json_body(resp).await,
        serde_json::json!(false),
        "nsfw defaults to false"
    );

    // Every whitelisted flag column is readable (defaulting to false). This pins the
    // whole validate_flag whitelist: dropping any name makes it an unknown flag (404)
    // instead of a valid boolean.
    for flag in [
        "unfinished",
        "outdated",
        "broken",
        "spoiler_unit_names",
        "blackholed",
    ] {
        let resp = c.get(flag_url(flag)).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "flag {flag} is readable");
        assert_eq!(
            json_body(resp).await,
            serde_json::json!(false),
            "{flag} defaults to false"
        );
    }

    // Writing requires a session...
    assert_eq!(
        set("nsfw", "true", None).await.unwrap().status(),
        StatusCode::UNAUTHORIZED,
        "set flag without login is 401"
    );
    // ...and ownership: a different account is forbidden.
    assert_eq!(
        set("nsfw", "true", Some(&intruder.cookie()))
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN,
        "a non-owner cannot set flags"
    );

    // The owner can set a flag, read it back, and clear it.
    assert_eq!(
        set("unfinished", "true", Some(&owner.cookie()))
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "owner sets the flag"
    );
    assert_eq!(
        json_body(c.get(flag_url("unfinished")).send().await.unwrap()).await,
        serde_json::json!(true)
    );
    assert_eq!(
        set("unfinished", "false", Some(&owner.cookie()))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        json_body(c.get(flag_url("unfinished")).send().await.unwrap()).await,
        serde_json::json!(false)
    );

    // An unknown flag name is 404 on both read and write.
    assert_eq!(
        c.get(flag_url("bogusflag")).send().await.unwrap().status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        set("bogusflag", "true", Some(&owner.cookie()))
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND,
        "unknown flag is 404 even for the owner"
    );
}

/// Tags: public read, owner-only write. `set` replaces the whole set; `addtags`
/// appends. Non-owners are forbidden.
#[tokio::test]
async fn map_tags_set_add_and_ownership() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "tagowner").await;
    let intruder = register(&c, &h, "tagintruder").await;
    let id = upload_map(&c, &h, Some(&owner), "e2etags.scx").await;

    let tags_url = h.url(&format!("/api/tags/{id}"));
    let addtags_url = h.url(&format!("/api/addtags/{id}"));

    // Read is public and returns an array (upload seeds an autogen tag).
    let resp = c.get(&tags_url).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(json_body(resp).await.is_array());

    // Writing needs auth + ownership.
    assert_eq!(
        c.post(&tags_url)
            .header("content-type", "application/json")
            .body(tags_body(&[("a", "1")]))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        c.post(&tags_url)
            .header("cookie", intruder.cookie())
            .header("content-type", "application/json")
            .body(tags_body(&[("a", "1")]))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );

    // The owner replaces all tags with exactly these two.
    assert_eq!(
        c.post(&tags_url)
            .header("cookie", owner.cookie())
            .header("content-type", "application/json")
            .body(tags_body(&[("genre", "aeon"), ("difficulty", "hard")]))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    let got = json_body(c.get(&tags_url).send().await.unwrap()).await;
    let mut kv: Vec<(String, String)> = got
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            (
                t["key"].as_str().unwrap().to_string(),
                t["value"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    kv.sort();
    assert_eq!(
        kv,
        vec![
            ("difficulty".to_string(), "hard".to_string()),
            ("genre".to_string(), "aeon".to_string()),
        ],
        "set replaces all tags (autogen tag included)"
    );

    // addtags appends without dropping the existing ones.
    assert_eq!(
        c.post(&addtags_url)
            .header("cookie", owner.cookie())
            .header("content-type", "application/json")
            .body(tags_body(&[("players", "8")]))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    let got = json_body(c.get(&tags_url).send().await.unwrap()).await;
    let keys: BTreeSet<String> = got
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["key"].as_str().unwrap().to_string())
        .collect();
    assert!(
        keys.contains("players") && keys.contains("genre") && keys.contains("difficulty"),
        "addtags appends to the existing tags: {got}"
    );
}

/// Flagging a map NSFW hides its details from anonymous users (403) while any
/// logged-in user can still see it.
#[tokio::test]
async fn nsfw_map_is_hidden_from_anonymous_users() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "nsfwowner").await;
    let id = upload_map(&c, &h, Some(&owner), "e2ensfw.scx").await;

    let info_url = h.url(&format!("/api/uiv2/map_info/{id}"));
    let minimap_url = h.url(&format!("/api/uiv2/minimap/{id}"));

    // Before flagging, an anonymous client can view the details.
    assert_eq!(
        c.get(&info_url).send().await.unwrap().status(),
        StatusCode::OK,
        "an sfw map is visible to anonymous users"
    );

    // The owner flags it NSFW.
    assert_eq!(
        c.post(h.url(&format!("/api/flags/{id}/nsfw")))
            .header("cookie", owner.cookie())
            .header("content-type", "application/json")
            .body("true")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );

    // Anonymous is now forbidden from details and the minimap...
    assert_eq!(
        c.get(&info_url).send().await.unwrap().status(),
        StatusCode::FORBIDDEN,
        "anonymous is blocked from an NSFW map's details"
    );
    assert_eq!(
        c.get(&minimap_url).send().await.unwrap().status(),
        StatusCode::FORBIDDEN,
        "anonymous is blocked from an NSFW map's minimap"
    );

    // ...but any logged-in user can still view it.
    assert_eq!(
        c.get(&info_url)
            .header("cookie", owner.cookie())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "a logged-in user can view the NSFW map"
    );
}

/// Search returns the uploaded map for a query matching its (indexed) filename,
/// and does not return it for a disjoint query. The map is searchable immediately
/// because denormalization runs inside the upload transaction.
#[tokio::test]
async fn search_returns_uploaded_map_for_matching_query_only() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "searchowner").await;
    // The filename is indexed for search; embed a distinctive token in it.
    let id = upload_map(&c, &h, Some(&owner), "haystackmarker.scx").await;

    // A matching query returns our map...
    assert!(
        search_ids(&c, &h, "haystackmarker").await.contains(&id),
        "a matching search should return the uploaded map"
    );
    // ...a disjoint query does not.
    assert!(
        !search_ids(&c, &h, "zzqxjjwwknomatchxyz")
            .await
            .contains(&id),
        "a non-matching search must not return the uploaded map"
    );
}

/// The four time-window bounds arrive from the client in **milliseconds** but the
/// DB stores times in **seconds**, so `search_cache` divides each by 1000 before
/// binding it as a SQL bound. This pins that conversion in BOTH query branches
/// (empty-query and keyword) against `/`→`*` and `/`→`%` mutations, by choosing
/// bounds where the correct `/1000` includes/excludes the uploaded map but the
/// mutated arithmetic flips it. The map's `uploaded_time` is `now()` and its
/// `modified_time` is 1_700_000_000 s (from the upload's `last_modified`); both sit
/// between 1e6 s and 2e9 s, which the crafted bounds below rely on.
#[tokio::test]
async fn search_time_bounds_divide_millis_to_seconds() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "timeboundowner").await;
    let id = upload_map(&c, &h, Some(&owner), "timeboundmarker.scx").await;

    // (extra query params) -> whether the map should still be returned.
    let cases: &[(&str, bool)] = &[
        // Default bounds: the map is in range → returned. A `%` mutation collapses a
        // large "before" bound to ~0 (`n*1000` ms ends in 000, so `% 1000 == 0`),
        // excluding everything — so this case also guards the before-bound `%`.
        ("", true),
        // Upper ("before") bounds: correct /1000 puts the map ABOVE a tiny
        // 1e9 ms (= 1e6 s) cap → excluded; `*1000` balloons the cap → wrongly kept.
        ("time_uploaded_before=1000000000", false),
        ("last_modified_before=1000000000", false),
        // Lower ("after") bounds, the `*` side: correct /1000 leaves the map ABOVE a
        // tiny 1e9 ms floor → kept; `*1000` lifts the floor past the map → dropped.
        ("time_uploaded_after=1000000000", true),
        ("last_modified_after=1000000000", true),
        // Lower ("after") bounds, the `%` side: a huge 2e12 ms (= 2e9 s) floor is
        // above the map → excluded; `% 1000 == 0` would drop the floor → wrongly kept.
        ("time_uploaded_after=2000000000000", false),
        ("last_modified_after=2000000000000", false),
    ];

    // Both the empty-query branch (lines 106-109) and the keyword branch (191-194)
    // build the same bounds; run every case through each.
    for query in ["", "timeboundmarker"] {
        for (params, expect_present) in cases {
            let url = h.url(&format!("/api/uiv2/search/{query}?{params}"));
            let resp = c.get(&url).send().await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK, "search {url}");
            let present = json_body(resp).await["maps"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["id"].as_str() == Some(id.as_str()));
            assert_eq!(
                present, *expect_present,
                "query={query:?} params={params:?}: expected map present = {expect_present}"
            );
        }
    }
}

/// The seven CHK endpoints all serve, purely from Postgres, the map parsed at
/// upload — no gsfs/Backblaze needed. Covers the JSON views (strings, riff chunks,
/// section object, trig/mbrf/eups arrays) and the raw `download_chk`, whose bytes
/// re-hash to the requested hash. Also pins the app's "missing → 500" behavior.
#[tokio::test]
async fn chk_endpoints_serve_the_parsed_map() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "chkowner").await;
    let id = upload_map(&c, &h, Some(&owner), "e2echk.scx").await;

    // strings: a non-empty JSON array of the map's scenario strings.
    let strings = json_body(
        c.get(h.url(&format!("/api/chk/strings/{id}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert!(
        strings.as_array().is_some_and(|a| !a.is_empty()),
        "strings is a non-empty array: {strings}"
    );

    // riff_chunks: metadata objects (size + offset), no chunk bytes.
    let chunks = json_body(
        c.get(h.url(&format!("/api/chk/riff_chunks/{id}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    let arr = chunks.as_array().expect("riff_chunks is an array");
    assert!(!arr.is_empty(), "a real map has RIFF chunks");
    assert!(
        arr[0].get("size").is_some() && arr[0].get("offset").is_some(),
        "each chunk carries size + offset: {}",
        arr[0]
    );

    // json: section-keyed object that includes core sections and omits the ones
    // the serializer deliberately drops (TRIG/TILE/MTXM/ISOM).
    let sections = json_body(
        c.get(h.url(&format!("/api/chk/json/{id}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    let obj = sections.as_object().expect("chk/json is an object");
    assert!(obj.contains_key("DIM"), "chk/json includes DIM: {sections}");
    for excluded in ["TRIG", "TILE", "MTXM", "ISOM"] {
        assert!(!obj.contains_key(excluded), "chk/json omits {excluded}");
    }

    // trig / mbrf / eups are JSON arrays (typically empty for a normal map).
    for path in ["trig", "mbrf", "eups"] {
        let resp = c
            .get(h.url(&format!("/api/chk/{path}/{id}")))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "GET chk/{path}");
        assert!(json_body(resp).await.is_array(), "chk/{path} is an array");
    }
    // The fixture is a normal map: no unit has an out-of-range owner (>12) or unit
    // id (>227), so the eups (extended-unit) set is empty. Pinning it empty catches
    // a widened comparison (`>`→`<`) that would sweep in ordinary units.
    assert_eq!(
        json_body(
            c.get(h.url(&format!("/api/chk/eups/{id}")))
                .send()
                .await
                .unwrap()
        )
        .await,
        serde_json::json!([]),
        "a normal map has no extended-unit placements"
    );

    // download_chk: raw bytes whose sha256 reproduces the requested hash.
    let internal_id = map_internal_id(&c, &h, &id).await;
    let chkhash = h
        .db_text("select chkblob from map where id = $1", internal_id)
        .await;
    let resp = c
        .get(h.url(&format!("/api/chk/{chkhash}")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "download_chk is 200");
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/octet-stream")
    );
    let bytes = resp.bytes().await.unwrap();
    use sha2::{Digest, Sha256};
    let rehash = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(
        rehash, chkhash,
        "download_chk body re-hashes to the requested chk hash"
    );

    // An unknown chk hash is a 500 (query_one, no 404-for-missing mapping).
    assert_eq!(
        c.get(h.url("/api/chk/deadbeefnope"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "an unknown chk hash is a 500"
    );
}

/// `GET /api/chk/eups/{id}` returns the placed units that are "extended" — owner
/// out of 0..=11 (`owner > 12`) OR unit id out of 0..=227 (`unit_id > 227`). Normal
/// maps have none (pinned empty in `chk_endpoints_serve_the_parsed_map`), so the two
/// `>` comparisons in the filter go untested there. These EUP fixtures each populate
/// exactly one side of the `||`: the counts (39 owner-side, 7 unit-id-side) pin both
/// comparisons — a `>`→`==` collapse on either would drop that fixture's whole set.
#[tokio::test]
async fn chk_eups_lists_extended_unit_placements() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "eupowner").await;

    // Owner-extended (owner > 12, unit_id <= 227): 39 placements.
    let poker = upload_fixture(
        &c,
        &h,
        Some(&owner),
        "poker_defense.scx",
        POKER_DEFENSE_SHA256,
        POKER_DEFENSE_LEN,
    )
    .await;
    let eups = json_body(
        c.get(h.url(&format!("/api/chk/eups/{poker}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        eups.as_array().map(Vec::len),
        Some(39),
        "the owner-extended EUP map lists 39 extended placements, got {eups}"
    );

    // Unit-id-extended (unit_id > 227, owner in range): 7 placements.
    let untitled = upload_fixture(
        &c,
        &h,
        Some(&owner),
        "untitled_eup.scx",
        UNTITLED_EUP_SHA256,
        UNTITLED_EUP_LEN,
    )
    .await;
    let eups = json_body(
        c.get(h.url(&format!("/api/chk/eups/{untitled}")))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        eups.as_array().map(Vec::len),
        Some(7),
        "the unit-id-extended EUP map lists 7 extended placements, got {eups}"
    );
}

/// The minimap image endpoints serve the PNG rendered synchronously at upload.
/// This also pins two deliberate cross-endpoint inconsistencies: an NSFW map
/// blocks anonymous callers with **403** on the uiv2 route but **401** on the core
/// route, and an unknown id is **404** on the chk-hash (access-checked) route but
/// **500** on the map-id (query_one) route.
#[tokio::test]
async fn minimap_endpoints_serve_png_and_gate_access() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "mmowner").await;
    let id = upload_map(&c, &h, Some(&owner), "e2eminimap.scx").await;
    let internal_id = map_internal_id(&c, &h, &id).await;
    let chkhash = h
        .db_text("select chkblob from map where id = $1", internal_id)
        .await;

    let is_png = |resp: &reqwest::Response| {
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            == Some("image/png")
    };

    // uiv2 minimap (keyed by map id): 200 PNG for an sfw map, anonymously.
    let resp = c
        .get(h.url(&format!("/api/uiv2/minimap/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "uiv2 minimap is 200");
    assert!(is_png(&resp), "uiv2 minimap is a PNG");

    // core minimap + resized (keyed by chk hash): 200 PNG.
    for path in [
        format!("/api/minimap/{chkhash}"),
        format!("/api/minimap_resized/{chkhash}"),
    ] {
        let resp = c.get(h.url(&path)).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "GET {path}");
        assert!(is_png(&resp), "{path} is a PNG");
    }

    // Unknown id: 404 on the access-checked chk route, 500 on the query_one route.
    assert_eq!(
        c.get(h.url("/api/minimap/deadbeefnope"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND,
        "unknown chk hash is a 404"
    );
    assert_eq!(
        c.get(h.url("/api/uiv2/minimap/9999999"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "unknown map id is a 500"
    );

    // Flag the map NSFW; anonymous access now diverges by endpoint family.
    assert_eq!(
        c.post(h.url(&format!("/api/flags/{id}/nsfw")))
            .header("cookie", owner.cookie())
            .header("content-type", "application/json")
            .body("true")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        c.get(h.url(&format!("/api/uiv2/minimap/{id}")))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN,
        "uiv2 minimap blocks anonymous NSFW with 403"
    );
    assert_eq!(
        c.get(h.url(&format!("/api/minimap/{chkhash}")))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED,
        "core minimap blocks anonymous NSFW with 401"
    );
}

/// The search-result popup returns the scenario name plus a base64 PNG minimap
/// with a 60s cacheable header; an NSFW map blocks anonymous callers (401).
#[tokio::test]
async fn search_result_popup_returns_scenario_and_minimap() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "popupowner").await;
    let id = upload_map(&c, &h, Some(&owner), "e2epopup.scx").await;

    let resp = c
        .get(h.url(&format!("/api/search_result_popup/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok()),
        Some("public, max-age=60, immutable")
    );
    let body = json_body(resp).await;
    assert!(
        body["scenario"].is_string(),
        "popup carries the scenario name: {body}"
    );
    assert!(
        body["minimap"].as_str().is_some_and(|m| !m.is_empty()),
        "popup carries a non-empty base64 minimap"
    );

    // Flag NSFW → the anonymous popup is 401.
    assert_eq!(
        c.post(h.url(&format!("/api/flags/{id}/nsfw")))
            .header("cookie", owner.cookie())
            .header("content-type", "application/json")
            .body("true")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        c.get(h.url(&format!("/api/search_result_popup/{id}")))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED,
        "anonymous popup for an NSFW map is 401"
    );
}

/// Blackholing a map hides it from everyone except its owner (and admins): an
/// anonymous *or* a logged-in non-owner request 404s on the details, the SSR page,
/// and the popup, while the owner still sees it. Stronger than the NSFW gate,
/// which only blocks anonymous users.
#[tokio::test]
async fn blackholed_map_is_hidden_from_non_owners() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "bhowner").await;
    let intruder = register(&c, &h, "bhintruder").await;
    let id = upload_map(&c, &h, Some(&owner), "e2eblackhole.scx").await;

    // Visible to an anonymous client before blackholing.
    assert_eq!(
        c.get(h.url(&format!("/api/uiv2/map_info/{id}")))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );

    // The owner blackholes it.
    assert_eq!(
        c.post(h.url(&format!("/api/flags/{id}/blackholed")))
            .header("cookie", owner.cookie())
            .header("content-type", "application/json")
            .body("true")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );

    // Anonymous AND a logged-in non-owner now get 404 across every surface.
    for (label, cookie) in [("anonymous", None), ("non-owner", Some(intruder.cookie()))] {
        for path in [
            format!("/api/uiv2/map_info/{id}"),
            format!("/map/{id}"),
            format!("/api/search_result_popup/{id}"),
        ] {
            let mut req = c.get(h.url(&path));
            if let Some(ck) = &cookie {
                req = req.header("cookie", ck.clone());
            }
            assert_eq!(
                req.send().await.unwrap().status(),
                StatusCode::NOT_FOUND,
                "{label} is 404 on {path}"
            );
        }
    }

    // The owner still sees the details.
    assert_eq!(
        c.get(h.url(&format!("/api/uiv2/map_info/{id}")))
            .header("cookie", owner.cookie())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK,
        "the owner can still view a blackholed map"
    );
}

/// With a matching map present, the random endpoints return that map's web id as a
/// bare JSON string (never an HTTP redirect); an unknown sort is a 500.
#[tokio::test]
async fn random_returns_a_matching_map_id() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "randowner").await;
    // Distinctive filename token so the query-scoped random matches deterministically.
    let id = upload_map(&c, &h, Some(&owner), "e2erandommarker.scx").await;

    // It's the only (non-NSFW) map, so random must return exactly it, as a string.
    let body = json_body(c.get(h.url("/api/uiv2/random")).send().await.unwrap()).await;
    assert_eq!(
        body,
        serde_json::json!(id),
        "random returns the map's web id as a JSON string"
    );

    // Query-scoped random over a token in the filename returns the same map.
    let body = json_body(
        c.get(h.url("/api/uiv2/random/e2erandommarker"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(body, serde_json::json!(id));

    // An unknown sort value is a 500.
    assert_eq!(
        c.get(h.url("/api/uiv2/random?sort=bogus"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "an unknown sort is a 500"
    );
}

/// The read-only landing-page endpoints each run one query and JSON-encode the
/// rows: `/api/uiv2/{featured,last_viewed,last_downloaded,last_uploaded}_maps`,
/// `most_{viewed,downloaded}_maps`, and `last_uploaded_replays`. A whole-body
/// mutation (`-> Ok(Json(vec![]))`) survives unless a test proves the endpoint
/// returns real rows, so we arrange for one map to satisfy every query and assert
/// each endpoint surfaces it.
///
/// One uploaded map, made to qualify for all of them: viewing it (via `map_info`)
/// sets `last_viewed`; the rest need state no HTTP route creates, seeded directly —
/// a `last_downloaded` timestamp, a `featuredmaps` row, and a `replay` joined to
/// the map by chk hash.
#[tokio::test]
async fn landing_endpoints_surface_maps_and_replays() {
    let h = Harness::start().await;
    let c = client();
    let owner = register(&c, &h, "landingowner").await;
    let id = upload_map(&c, &h, Some(&owner), "e2elanding.scx").await;
    let internal = map_internal_id(&c, &h, &id).await;

    // View it once so `last_viewed` (and the view count) is populated.
    let resp = c
        .get(h.url(&format!("/api/uiv2/map_info/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "map_info view");

    // Seed the state no HTTP route creates: a download timestamp, a featured-map
    // entry, and a replay whose chk hash matches the map (so the replay↔map join in
    // last_uploaded_replays succeeds).
    let chkblob = h
        .db_text("select chkblob from map where id = $1", internal)
        .await;
    h.db_execute(&format!(
        "update map set downloads = 42, last_downloaded = 1700000001 where id = {internal};
         insert into featuredmaps (map_id, rank) values ({internal}, 100);
         insert into replayblob (hash, data) values ('e2ereplayhash', '\\x00');
         insert into replay (id, hash, uploaded_by, uploaded_time, chkhash)
             values (1, 'e2ereplayhash', null, 1700000002, '{chkblob}');"
    ))
    .await;

    // Every map-returning endpoint should include our map by web id.
    for path in [
        "featured_maps",
        "last_viewed_maps",
        "last_downloaded_maps",
        "last_uploaded_maps",
        "most_viewed_maps",
        "most_downloaded_maps",
    ] {
        let resp = c
            .get(h.url(&format!("/api/uiv2/{path}")))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "GET /api/uiv2/{path}");
        let body = json_body(resp).await;
        let arr = body.as_array().expect("array body");
        assert!(
            arr.iter()
                .any(|m| m["map_id"].as_str() == Some(id.as_str())),
            "/api/uiv2/{path} should surface our map, got {body}"
        );
    }

    // The replay endpoint should surface the seeded replay, keyed to our map.
    let resp = c
        .get(h.url("/api/uiv2/last_uploaded_replays"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "GET last_uploaded_replays");
    let body = json_body(resp).await;
    let arr = body.as_array().expect("array body");
    assert!(
        arr.iter()
            .any(|r| r["map_id"].as_str() == Some(id.as_str())),
        "last_uploaded_replays should surface the replay for our map, got {body}"
    );
}
