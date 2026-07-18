//! Test harness + HTTP helpers for the end-to-end suite (`tests/e2e.rs`).
//!
//! ## How it's wired
//!
//! `make e2e` starts **one shared Postgres** (the project's `postgres/` image —
//! pgvector + the `00-init.sh` bootstrap that builds the schema into the
//! `bounding.net` database) and passes its address in via `E2E_PG_HOST` /
//! `E2E_PG_PORT` / `E2E_PG_PASSWORD`. The container's lifecycle is the Makefile's
//! job, not the tests'.
//!
//! Each test gets an **isolated copy** of that schema via a Postgres *template
//! database*: `CREATE DATABASE <unique> TEMPLATE "bounding.net"` is a fast
//! copy-on-write-ish clone, so a test starts from the pristine schema with its own
//! data *and* its own app process (hence its own in-memory rate-limiter state).
//! That isolation is why these tests can run in parallel and be written
//! independently — no shared-DB bookkeeping. Each test drops its own database on
//! teardown (see [`Harness::drop`]) so one long-lived container survives the many
//! runs of a `make mutants` sweep; the container itself is discarded by `make e2e`.
//!
//! `E2E_PG_*` is mandatory: a test run without it (i.e. not launched via
//! `make e2e`) fails loudly rather than skipping, so a misconfigured run can never
//! masquerade as a green suite.

use std::fs;
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use reqwest::header::SET_COOKIE;
use reqwest::{Client, Response};

/// Role the app authenticates as (created by `00-init.sh`); also owns the schema.
const APP_USER: &str = "bounding.net";
/// The schema database `00-init.sh` builds — used as the per-test template.
const TEMPLATE_DB: &str = "bounding.net";

// ---------------------------------------------------------------------------
// Shared-Postgres connection info, provisioned by `make e2e`.
// ---------------------------------------------------------------------------

struct PgEnv {
    host: String,
    port: u16,
    password: String,
}

fn pg_env() -> PgEnv {
    PgEnv {
        host: std::env::var("E2E_PG_HOST")
            .expect("E2E_PG_HOST not set — run the E2E tests via `make e2e`"),
        port: std::env::var("E2E_PG_PORT")
            .expect("E2E_PG_PORT not set — run the E2E tests via `make e2e`")
            .parse()
            .expect("E2E_PG_PORT must be a valid port number"),
        password: std::env::var("E2E_PG_PASSWORD")
            .expect("E2E_PG_PASSWORD not set — run the E2E tests via `make e2e`"),
    }
}

impl PgEnv {
    /// Connection string for the superuser (used to create per-test databases).
    fn admin_conn(&self, dbname: &str) -> String {
        format!(
            "host={} port={} user=postgres password={} dbname={dbname}",
            self.host, self.port, self.password
        )
    }
}

fn unique_db_name() -> String {
    static N: AtomicU64 = AtomicU64::new(0);
    format!(
        "e2e_{}_{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    )
}

// ---------------------------------------------------------------------------
// Per-test harness: an isolated template-copied DB + its own server process.
// ---------------------------------------------------------------------------

pub struct Harness {
    app: Child,
    base: String,
    log_path: PathBuf,
    work_dir: PathBuf,
    db: String,
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = self.app.kill();
        let _ = self.app.wait();

        // Drop this test's database. A single `make e2e` could leave them for the
        // container teardown to reap, but a `make mutants` run re-runs the whole
        // suite once per mutant against ONE long-lived Postgres — leaking a
        // template-cloned database every test would exhaust its disk/catalog. Run
        // it on a dedicated thread with its own runtime: `Drop` is synchronous and
        // may fire inside the test's tokio runtime, where `block_on` panics.
        let db = self.db.clone();
        let _ = std::thread::spawn(move || {
            let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            rt.block_on(async move {
                let pg = pg_env();
                if let Ok((client, connection)) =
                    tokio_postgres::connect(&pg.admin_conn("postgres"), tokio_postgres::NoTls).await
                {
                    let conn_task = tokio::spawn(async move {
                        let _ = connection.await;
                    });
                    // FORCE terminates any backend still attached (the app is
                    // killed above, but its pooled connections may not have closed).
                    let _ = client
                        .execute(
                            &format!(r#"DROP DATABASE IF EXISTS "{db}" WITH (FORCE)"#),
                            &[],
                        )
                        .await;
                    drop(client);
                    let _ = conn_task.await;
                }
            });
        })
        .join();

        let _ = fs::remove_file(&self.log_path);
        let _ = fs::remove_dir_all(&self.work_dir);
    }
}

impl Harness {
    /// Provision an isolated DB + server. Panics (failing the test) when the
    /// shared Postgres isn't provisioned — i.e. not launched via `make e2e`.
    pub async fn start() -> Harness {
        let pg = pg_env();

        let db = unique_db_name();
        create_template_db(&pg, &db).await;

        let root = repo_root();

        // Give each app its own working dir under the system temp dir. The server
        // writes its `./pending/` blob-staging dirs relative to cwd; keeping cwd in
        // /tmp means those land there — removed on Drop, and swept on the next
        // reboot — instead of polluting the repo working tree. Symlink in the asset
        // dirs the server reads relative to cwd (notably ./dist/.vite/manifest.json,
        // read at startup); ROOT_DIR is passed as an absolute path, so it's fine.
        let work_dir = std::env::temp_dir().join(format!("scmscx-e2e-{db}"));
        let _ = fs::remove_dir_all(&work_dir);
        fs::create_dir_all(&work_dir).unwrap();
        // The server reads `./dist/.vite/manifest.json` at render time and serves
        // `./dist/assets`. Point `./dist` at the real vite build output
        // (`dist/vite`, which `make e2e` guarantees is built and carries the
        // populated manifest) rather than the top-level `dist/`, whose committed
        // `.vite/manifest.json` is an empty `{}` stub — with the stub every SSR
        // handler panics on `manifest.get(...).unwrap()`. This mirrors production,
        // where the Dockerfile's `COPY dist/assets .` makes the runtime `./dist`
        // the built output too. `public/` (served at `/`) is linked as-is.
        let dist_src = {
            let built = root.join("dist/vite");
            if built.join(".vite/manifest.json").exists() {
                built
            } else {
                root.join("dist")
            }
        };
        if dist_src.exists() {
            std::os::unix::fs::symlink(&dist_src, work_dir.join("dist")).unwrap();
        }
        if root.join("public").exists() {
            std::os::unix::fs::symlink(root.join("public"), work_dir.join("public")).unwrap();
        }

        let log_path = std::env::temp_dir().join(format!("scmscx-e2e-app-{db}.log"));
        let log = fs::File::create(&log_path).unwrap();
        let log_err = log.try_clone().unwrap();

        // Bind the app and Prometheus ports *here* and hand the live listening
        // sockets down to the child as inherited file descriptors, rather than
        // picking a port, closing it, and hoping the child re-binds it before
        // anything else grabs it. The parent holds each port continuously from
        // bind through hand-off, so there is no race window and no flaky
        // `AddrInUse` at startup. The server adopts these via
        // `common::telemetry::take_listener_from_env`.
        let app_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let prom_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let app_port = app_listener.local_addr().unwrap().port();
        let app_fd = app_listener.as_raw_fd();
        let prom_fd = prom_listener.as_raw_fd();

        // Prod mode, Backblaze/GSFS off, a small pool (parallel tests share one
        // Postgres), and inherited listener fds so instances never race.
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_scmscx-com"));
        cmd.current_dir(&work_dir) // ./dist symlinked in; ./pending/ stays in /tmp
            .env("BIND_FD", app_fd.to_string())
            .env("PROMETHEUS_FD", prom_fd.to_string())
            .env("DB_HOST", &pg.host)
            .env("DB_PORT", pg.port.to_string())
            .env("DB_USER", APP_USER)
            .env("DB_DATABASE", &db)
            .env("DB_PASSWORD", &pg.password)
            .env("DB_CONNECTIONS", "4")
            .env("BACKBLAZE_DISABLED", "true")
            .env("ROOT_DIR", root.join("app/web"))
            .env("RUST_LOG", "warn")
            .env_remove("DEV_MODE")
            .env_remove("GSFSFE_ENDPOINT")
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(log_err));

        // Rust marks socket fds `FD_CLOEXEC`, so they would close on `exec`. Clear
        // it on our two fds in a `pre_exec` hook — which runs post-fork in the
        // child only, so a sibling harness spawning concurrently never inherits
        // (and thus never keeps alive) our sockets.
        unsafe {
            cmd.pre_exec(move || {
                for fd in [app_fd, prom_fd] {
                    if libc::fcntl(fd, libc::F_SETFD, 0) == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }

        let mut app = cmd.spawn().expect("spawn scmscx-com");

        // The child inherited its own copies of the sockets (same underlying
        // listeners); drop the parent's so only the child holds them — the parent
        // never accepts on its side.
        drop(app_listener);
        drop(prom_listener);

        let base = format!("http://127.0.0.1:{app_port}");
        let ping = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            // Fail fast if the child exited during startup (e.g. a DB error)
            // instead of polling a dead process until the deadline.
            if let Ok(Some(_)) = app.try_wait() {
                let logs = fs::read_to_string(&log_path).unwrap_or_default();
                let _ = app.wait();
                panic!("server exited during startup.\n--- server log ---\n{logs}");
            }

            let up = ping
                .get(format!("{base}/sitemap.txt"))
                .send()
                .await
                .is_ok_and(|r| r.status().is_success());
            if up {
                break;
            }
            if Instant::now() >= deadline {
                let logs = fs::read_to_string(&log_path).unwrap_or_default();
                let _ = app.kill();
                let _ = app.wait();
                panic!("server did not become ready within 30s.\n--- server log ---\n{logs}");
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        }

        Harness {
            app,
            base,
            log_path,
            work_dir,
            db,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    /// The server's cwd-relative `./pending/` blob-staging dir (absolute path),
    /// where uploads land before the delivery pumpers ship them.
    pub fn pending_dir(&self) -> PathBuf {
        self.work_dir.join("pending")
    }

    /// Read a single text column from this test's database (as the superuser),
    /// binding one `i64` parameter. This is a fixture affordance, not a test
    /// oracle: it exists only to fetch opaque values the HTTP API never surfaces
    /// — notably a map's `chkblob` hash, which the chk-hash-keyed endpoints
    /// (`/api/chk/{hash}`, `/api/minimap/{chk_hash}`) take in the URL.
    pub async fn db_text(&self, sql: &str, param: i64) -> String {
        let pg = pg_env();
        let (client, connection) =
            tokio_postgres::connect(&pg.admin_conn(&self.db), tokio_postgres::NoTls)
                .await
                .expect("connect to per-test db");
        let conn_task = tokio::spawn(async move {
            let _ = connection.await;
        });
        let row = client
            .query_one(sql, &[&param])
            .await
            .expect("db_text query_one");
        let value: String = row.get(0);
        drop(client);
        let _ = conn_task.await;
        value
    }

    /// Run one or more statements against this test's database (as the superuser).
    /// Like [`Harness::db_text`], a fixture affordance rather than a test oracle:
    /// it seeds state that no HTTP endpoint can create — a curated `featuredmaps`
    /// row, a `replay`, a `last_downloaded` timestamp — so the read-only landing
    /// endpoints have something to return. The SQL is test-authored (never user
    /// input), so formatting our own ids into it is safe.
    pub async fn db_execute(&self, sql: &str) {
        let pg = pg_env();
        let (client, connection) =
            tokio_postgres::connect(&pg.admin_conn(&self.db), tokio_postgres::NoTls)
                .await
                .expect("connect to per-test db");
        let conn_task = tokio::spawn(async move {
            let _ = connection.await;
        });
        client.batch_execute(sql).await.expect("db_execute");
        drop(client);
        let _ = conn_task.await;
    }
}

/// `CREATE DATABASE <name> TEMPLATE bounding.net` — a fast isolated clone of the
/// schema for one test. (The name is generated, so string-formatting it into the
/// DDL — which can't take bind parameters — is safe.)
async fn create_template_db(pg: &PgEnv, name: &str) {
    let (client, connection) =
        tokio_postgres::connect(&pg.admin_conn("postgres"), tokio_postgres::NoTls)
            .await
            .expect("connect to postgres as superuser");
    let conn_task = tokio::spawn(async move {
        let _ = connection.await;
    });
    client
        .execute(
            &format!(r#"CREATE DATABASE "{name}" OWNER "{APP_USER}" TEMPLATE "{TEMPLATE_DB}""#),
            &[],
        )
        .await
        .expect("create per-test template database");
    drop(client);
    let _ = conn_task.await;
}

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <repo>/crates/bwmapserver
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repo root")
}

// ---------------------------------------------------------------------------
// HTTP helpers.
// ---------------------------------------------------------------------------

/// Client that neither follows redirects (so we can assert 3xx) nor stores
/// cookies (the app sets `Secure` cookies we drive back by hand over plain HTTP).
pub fn client() -> Client {
    Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap()
}

pub fn set_cookies(resp: &Response) -> Vec<String> {
    resp.headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .map(str::to_string)
        .collect()
}

/// The value of a `Set-Cookie` on the response, by name (e.g. `token`).
pub fn cookie_value(resp: &Response, name: &str) -> Option<String> {
    set_cookies(resp).into_iter().find_map(|c| {
        let pair = c.split(';').next()?.trim().to_string();
        let (k, v) = pair.split_once('=')?;
        (k == name).then(|| v.to_string())
    })
}

pub async fn json_body(resp: Response) -> serde_json::Value {
    let text = resp.text().await.unwrap();
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("expected JSON body, got {text:?}: {e}"))
}

/// POST `/api/uiv2/is_session_valid`, optionally carrying auth cookies.
pub async fn is_session_valid(c: &Client, url: &str, cookie: Option<&str>) -> Response {
    let mut req = c.post(url);
    if let Some(cookie) = cookie {
        req = req.header("cookie", cookie);
    }
    req.send().await.unwrap()
}
