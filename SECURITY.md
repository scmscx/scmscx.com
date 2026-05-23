# scmscx.com HTTP security audit

Static review of the HTTP surface exposed by `crates/bwmapserver` (≈60 routes).
This audit does not include dependency CVE scanning (`cargo audit`) or fuzzing
of the binary parsers reachable from upload (`bwmap`, `bwmpq`,
`stormlib-bindings`, `chkdraft-bindings`); both are worth a separate pass.

## Scope

All routes registered in `crates/bwmapserver/src/actix.rs::start`, plus the
middleware stack:

- `CacheHtmlTransformer`
- `PostgresLoggingTransformer`
- `UserSessionTransformer`
- `LanguageTransformer`
- `TrackingAnalyticsTransformer`
- `TraceIDTransformer`

Auth helpers reviewed: `bwcommon::check_auth4`, `crate::middleware::usersession`.
Auth-related DB code reviewed: `crate::db::{login, register, change_password,
check_password, change_username, set_tags, add_tags}`.

## Findings summary

| # | Severity | Title |
|---|---|---|
| 1 | Critical | Password hashing is single-round SHA-256 (no key stretching) — fixed (Argon2id, lazy-migrate legacy on login) |
| 2 | Critical | Session tokens stored plaintext and re-logged on every request |
| 3 | Critical | `change_password` does not require the current password |
| 4 | Critical | No rate limiting / lockout on `/api/login` or `/api/register` — fixed (per-IP + per-username, in-process) |
| 5 | High | Unbounded upload size + leaked partial uploads |
| 6 | High | Dev-mode default proxy is an SSRF surface if leaked into prod |
| 7 | High | Authorization gaps (replays, chk, search_result_popup, hardcoded admins; tags / chk-id minimap / flags / `tests/all_maps` fixed) |
| 8 | High | Username enumeration via `/api/login` timing |
| 9 | Medium | Non-constant-time token comparison |
| 10 | Medium | `check_auth4` uses `query_one` (500 DoS via cookie) — fixed, function deleted |
| 11 | Medium | Permanent cookies, no rotation, `secure` flag mismatch |
| 12 | Medium | `GET /api/logout` is CSRF-able |
| 13 | Medium | No CSRF tokens; SameSite=Lax is the only defense |
| 14 | Medium | Tracking-analytics fingerprint uses hardcoded salt |
| L1-L8 | Low / informational | Misc (see below) |

---

## Critical

### 1. Password hashing is single-round SHA-256 (no key stretching)

**Status:** fixed. Migrated to Argon2id with backwards-compatible verification
and lazy migration on successful login. No password resets required; users
can log in with their existing credentials and their stored hash is
transparently upgraded.

Schema change (in `postgres/00-init.sh`, plus the matching prod migration):
```sql
ALTER TABLE account
    ADD COLUMN password_algorithm text NOT NULL DEFAULT 'sha256-legacy';
```
The new column drives verifier dispatch; the existing `passwordhash` and
`salt` text columns are reused (now interpreted per-algorithm).

Implementation (`crates/bwmapserver/src/db.rs`):
- `hash_password(&str) -> Result<HashedPassword>`: generates a 16-byte
  random salt with `rand::rng().fill_bytes`, runs `Argon2::default()
  .hash_password_into(...)` (Argon2id, OWASP 2023 params: m=19456, t=2,
  p=1), and returns the triple `(algorithm="argon2id", salt=base64(salt),
  hash=base64(digest))` ready for the three columns.
- `verify_password(algorithm, password, username, salt, stored_hash) ->
  bool`: dispatches on the `password_algorithm` column. For `"argon2id"`,
  base64-decodes the salt and stored hash, recomputes via
  `hash_password_into`, and compares with a hand-rolled constant-time
  `ct_eq`. For `"sha256-legacy"`, recomputes the original `sha256_hex(
  username || password || salt)` and compares with `ct_eq`.
- `login(...)`: one query reads `(id, password_algorithm, passwordhash,
  salt, token)`. On success with a legacy entry, the row is re-hashed and
  the three columns are updated together (`update account set
  password_algorithm = 'argon2id', passwordhash = $new_hash_b64, salt =
  $new_salt_b64 where id = $1`). Migration failures are logged and
  swallowed.
- `register(...)`, `change_password(...)`, `change_username(...)`: always
  write Argon2id and all three columns. `change_username` keeps the
  password param because legacy hashes mix the username into the digest, so
  a username change without re-hashing would invalidate the legacy stored
  hash.
- `check_password(...)`: now one query (was two), uses `verify_password`.

Storage layout:

| Column              | Legacy row             | Argon2id row              |
|---------------------|------------------------|---------------------------|
| `password_algorithm`| `'sha256-legacy'`      | `'argon2id'`              |
| `salt`              | UUIDv4 hex (32 chars)  | base64(16-byte salt)      |
| `passwordhash`      | SHA-256 hex (64 chars) | base64(32-byte hash)      |

Side wins:
- `login` reduced from 2 queries to 1, which also closes the worst of the
  per-step timing variance from finding #8 (the remaining timing diff is
  argon2-verify-vs-no-verify on missing user; a dummy-hash patch can close
  that later).
- `check_password` reduced from 2 queries to 1.
- `change_password` no longer needs to fetch the username (legacy hash
  required it; Argon2 doesn't).

Operational notes:
- Run the `ALTER TABLE` above on existing deployments before rolling out
  the new code. The default of `'sha256-legacy'` means existing rows are
  classified correctly automatically.
- Argon2 default params (m=19MiB, t=2, p=1) are OWASP 2023 recommendations.
  Bump `Argon2::default()` to `Argon2::new(..., Params::new(...))` in
  `hash_password` if you want more conservative settings. Because params
  are fixed in code (not per-row), changing defaults will invalidate every
  existing Argon2 hash — you'd need to force a password reset or add a
  per-row params column at that point.
- Verification cost is now ~50–200 ms per attempt, which makes
  finding #4 (rate-limiting) higher-priority but also already makes online
  brute force much less feasible than the SHA-256 baseline.

### 2. Session tokens stored plaintext, and re-logged on every request

- `account.token` is a plaintext UUID (`db.rs:259-265, 291`). Anyone with read
  access to the `account` table can impersonate any user.
- `crates/bwmapserver/src/middleware/postgreslogging.rs:83-86, 122-124, 176, 194`
  writes the active session token AND the entire `Cookie:` header into
  `userlogs` for **every single request**:

  ```rust
  let user_token = req.extensions().get::<UserSession>().map(|x| x.token.clone());
  ...
  let cookies = header_map.get("cookie")...
  ... INSERT INTO userlogs (..., cookies, ..., user_token, ...) ...
  ```

A leaked `userlogs` backup → full session takeover for every user who hit the
site during the log window. Read access by any analytics/BI tooling has the
same effect.

**Fix:** Hash session tokens at rest (store `sha256(token)` in `account`,
compare against `sha256(cookie_token)` in `check_auth4`/`UserSessionTransformer`).
Stop logging `cookies` and `user_token`. If raw cookies are needed for
debugging, scrub the `token=` and `username=` cookies before insert.

### 3. `change_password` does not require the current password

`crates/bwmapserver/src/api/change_password.rs:9-46` only checks
`check_auth4` (cookie token), then writes the new password. It also does
**not** rotate `account.token`, so other sessions remain logged in with the
old cookie even after the password is changed.

Combined with finding #2: a stolen cookie → permanent account takeover that
the legitimate user cannot evict by changing their own password.

**Fix:** Require the current password on `/api/change-password`. Rotate
`account.token` on every successful password change (and on
`/api/change-username`, which already rewrites the hash). Provide a
"sign out everywhere" affordance.

### 4. No rate limiting / lockout on `/api/login` or `/api/register`

**Status:** fixed. Uses `governor 0.10.4` for the rate-limit logic plus
`actix-governor 0.10.0` for the per-IP middleware wiring
(`crates/bwmapserver/src/ratelimit.rs`).

Policy:
- **Per-IP, login:** 20 cells, refills at 20/min (GCRA). Counts every
  request.
- **Per-IP, register:** 3 cells, refills at 3/hour (one cell every
  20 min). Much tighter than login because legitimate users only
  register once; this is almost entirely a signup-spam defense.
- **Per-username, login:** 10 cells, refills at 10 per 15 min (one cell
  every 90 s). Counts every attempt — no reset on success, since
  governor's GCRA model doesn't carve out such a hook.
- Blocked per-IP returns `429` from the actix-governor middleware
  itself; per-username returns `429 Too Many Requests` with
  `Retry-After: <seconds>` (computed from governor's `NotUntil`) and
  the body `"Too many attempts. Please slow down."`.

Implementation notes:
- Each per-IP policy runs as middleware on its own scope containing
  only that route (`/api/login` or `/api/register`), so other routes
  are unaffected and the budgets are independent.
- Custom `RealIpKeyExtractor` (in `ratelimit.rs`) keys per-IP by
  `connection_info().realip_remote_addr()` (honors `X-Forwarded-For` /
  `Forwarded`), falling back to the direct peer. The default
  `PeerIpKeyExtractor` would map every request to the reverse proxy's
  IP, which would make the limiter useless behind the proxy.
- Per-username uses `governor::RateLimiter::keyed` directly via
  `web::Data<KeyedRateLimiter<String>>`. Middleware can't easily
  inspect the JSON body to extract the username, so this stays inline
  in the handler. The handler reads it via the standard
  `web::Data<...>` extractor and calls `check_key(&form.username)`.
- Both backends are lock-free (DashMap state); no Mutex on the hot
  path.
- Server restart resets state — acceptable for the single-host
  deployment this runs on.

Limitations / accepted trade-offs:
- An attacker can lock a known username out of login for ~15 minutes by
  burning through 10 attempts. Counter doesn't reset on successful
  login (governor's GCRA model doesn't offer that hook). The 15-min
  window is a nuisance, not a denial of service.
- State is per-process; if we ever scale beyond one host, two requests
  to different replicas split the budget. Move to a shared store
  (Postgres or Redis) at that point.
- The chosen Argon2 verification cost (~50–200 ms) combined with the
  per-IP limit makes online brute force impractical even before the
  per-username lockout kicks in.

`/api/change-password` and `/api/uiv2/upload-map` are not yet rate-limited;
they're lower-priority and depend on related fixes (#3 for change-password,
#5 for upload).

---

## High

### 5. Unbounded upload size + leaked partial uploads

`crates/bwmapserver/src/api/uiv2/upload.rs:40-95`

- `query.length` is supplied by the client. The handler enforces
  `total_file_size <= query.length` *during* the read, but no global
  server-side upper bound exists. A client can send `length=2^63-1` and
  stream forever, filling the disk.
- If `sha256hash != query.sha256` (or any later step fails), the temp file at
  `./pending/tmp/{uuid}.scx` is **never deleted** — `tokio::fs::remove_file`
  is only reached on the success path. Repeated bad uploads exhaust disk.
- `tokio::fs::copy(&fake_filename, fake_filename2)` doubles the disk usage
  per upload before cleanup.

**Fix:** Hard server-side max size (e.g. 32 MB) checked before reading any
body bytes. Wrap the temp file in a guard / RAII helper that deletes it on
every error path (or use `tempfile::NamedTempFile`). Drop the intermediate
copy: rename the original file in place after hashing is complete.

### 6. Dev-mode default proxy is an SSRF surface if leaked into prod

`crates/bwmapserver/src/actix.rs:1067-1088`

```rust
svc.default_service(web::to(|req, client| async move {
    let url = format!("http://localhost:3000{path_query}");
    client.get(&url).send().await? ...
}))
```

Only registered when `DEV_MODE=true`, but the flag comes from `is_dev_mode()`
which is just a `std::env::var("DEV_MODE")` read with no defense in depth.
If that env var ever leaks into a production deployment (typo in compose
file, copy/pasted Dockerfile, etc.), every unmatched URL becomes an
attacker-controlled GET against any localhost port.

**Fix:** Compile the proxy out of release builds (`#[cfg(debug_assertions)]`
or a Cargo feature). At minimum, restrict `path_query` to a strict allowlist
regex and refuse `..`.

### 7. Authorization gaps

Resolved (line numbers refer to pre-fix state):

- ~~`POST /api/addtags/{map_id}`~~ — fixed. `db::add_tags` now takes a
  `user_id` parameter and runs the same `map.uploaded_by = user_id || user_id == 4`
  check (inside the same transaction) that `set_tags` already had. Handler
  returns 403 on unauthorized.
- ~~`GET /api/minimap/{chk_id}`, `GET /api/minimap_resized/{chk_id}`~~ —
  fixed. Both handlers route through a `check_chk_access(pool, chk_id,
  user_id)` helper. A chk hash inherits the **most-restrictive** flag of
  any map that references it: if even one map is blackholed, the chk is
  blackholed; if any one is NSFW, the chk is NSFW. The helper runs a
  single aggregate query
  (`bool_or(blackholed)`, `bool_or(nsfw)`, `bool_or(true)`) and returns:
  - `404 Not Found` — no map references this chk, or any map referencing
    it is blackholed and the viewer isn't admin (id == 4). Blackholed is
    treated as deleted.
  - `401 Unauthorized` — any map referencing the chk is NSFW and the
    viewer isn't logged in. NSFW requires login.
  - `200 OK` otherwise.

  401/404 responses set `Cache-Control: no-cache` so an anonymous denial
  isn't replayed to a logged-in user.
- ~~`POST /api/flags/{map_id}/{flag}`~~ — fixed. The handler now inspects
  `con.execute(...)` row count and returns 403 when 0 rows match (i.e. the
  user is not the owner and not admin, or the map doesn't exist), instead
  of always returning 200.
- ~~`GET /api/tests/all_maps`~~ — removed entirely (route, handler,
  `api::tests` module, file).

Outstanding:

| Endpoint | Issue | File |
|---|---|---|
| `GET /api/replays/{replay_id}` | No NSFW / blackholed / ownership check; any integer ID downloads the raw replay blob. | `actix.rs:212-236` |
| `GET /api/chk/{strings,riff_chunks,json,trig,mbrf,eups}/{map_id}`, `GET /api/chk/{chk_hash}` | No NSFW / blackholed check; full CHK payload + strings/descriptions of restricted maps reachable. | `api/chk.rs` |
| `GET /api/search_result_popup/{map_id}` | Returns base64 minimap + scenario name without NSFW / blackholed gating. (Not in original list, surfaced while fixing the chk-id minimap routes.) | `actix.rs:372-421` |
| Hardcoded admin user IDs | `id == 4` (and 4/5/18/24/32 in `get_selection_of_random_maps`, 4/18/24/32 in nsfw variant) sprinkled across `actix.rs:494-499, 557-562`, `hacks.rs:269, 305`, `flags.rs`, `db.rs:178, 222`. Any new admin requires a code change; any reassigned uid silently grants admin. | multiple |

**Remaining fix:**
- Replays / chk-* / `search_result_popup` endpoints: require the same NSFW +
  blackholed gating that `/api/uiv2/map_info` uses, by joining to `map` on
  the chk hash (or by map_id where applicable).
- Move admin to a DB column (e.g. `account.is_admin BOOLEAN`) and check that
  everywhere instead of integer literals.

### 8. Username enumeration via `/api/login`

`db::login` first runs `select salt from account where username = $1`.
Username-not-found and password-wrong both surface as `query_one(...)`
errors, and the handler does return a generic message ("Either the username
does not exist or the password is incorrect"). However, response **timing**
differs significantly: not-found = one DB roundtrip; wrong-password = two DB
roundtrips + a SHA hash. With #4 in place this is exploitable.

**Fix:** Always do the dummy hash, then a single
`select token from account where username = $1 and passwordhash = $2`. If 0
rows, return the generic error. Equalize timing further with a sleep jitter
or constant-time wrap.

---

## Medium

### 9. Non-constant-time token comparison

`crates/bwmapserver/src/middleware/usersession.rs:122` and
`crates/bwcommon/src/common.rs:34`:

```rust
if cookie_token.value() == db_idtoken.1.as_str() { ... }
```

Cookie tokens are compared as plain strings using `==`. Theoretically
timing-leakable across the network for the first few bytes of the token.

**Fix:** Use `subtle::ConstantTimeEq` or compare hashes (which would also
address #2).

### 10. `check_auth4` uses `query_one` instead of `query_opt`

`crates/bwcommon/src/common.rs:25-30`. If a tampered `username` cookie names
a non-existent user, `query_one` returns `Err`, which `?`s up to a 500. An
attacker can cheaply force every authenticated request from a victim's
browser to 500 by setting a known-bad `username` cookie.

**Status:** fixed. All six handler callsites of `check_auth4` were
replaced with `req.extensions().get::<UserSession>()` (the
`UserSessionTransformer` middleware already populates it via `query_opt`),
and the now-dead `check_auth4` function and its `pub use` re-export were
deleted from `bwcommon`. The DoS vector is gone.

### 11. Permanent cookies, no rotation, `secure` flag mismatch

- `Cookie::build("token", ...)` and `Cookie::build("username", ...)` are
  marked `.permanent()` (~20 years) and never rotate. With #3, a lost device
  is effectively forever access.
- `register.rs:62`, `change_username.rs:58`, `logout.rs:13`, and the
  session-cleanup path in `middleware/usersession.rs:80, 91` hard-code
  `secure(true)`, while `login.rs:30` uses `secure(!is_dev_mode())`. Either
  dev cookies don't get set on the secure-true paths (silent failure on
  HTTP), or, if a prod env runs without TLS, login cookies are sent in
  cleartext. (`change_password.rs` doesn't set cookies — fine.)
- The `username` cookie is **not** `HttpOnly`. Any client-side XSS exfiltrates
  it (and combined with the `tac` fingerprint, that's enough to forge a full
  session in many cases).

**Fix:**
- Use shorter cookie lifetimes (e.g. 30 days) and rotate `account.token` on
  password / username change and on a "sign out everywhere" action.
- Make the `secure` flag consistent: `secure(!is_dev_mode())` everywhere, or
  unconditionally `secure(true)` if you no longer support HTTP dev.
- Mark `username` cookie `HttpOnly` (the frontend can read identity from
  `/api/uiv2/is_session_valid` and a small `/api/me` endpoint).

### 12. Logout via GET (CSRF logout)

`crates/bwmapserver/src/api/logout.rs:40` uses `#[get]`. Any third-party site
can log a user out via an `<img src="https://scmscx.com/api/logout">`.
Annoying, not damaging.

**Fix:** Switch to `#[post]` (the new `/api/uiv2/logout` route already
suffers the same issue — also POST it).

### 13. No CSRF tokens; SameSite=Lax is the only defense

All state-changing endpoints (`POST /api/login`, `register`,
`change-password`, `change-username`, `tags`, `addtags`, `flags`,
`upload-map`) rely solely on `SameSite=Lax`. `Lax` does block cross-origin
POSTs, so most CSRF is mitigated, but:

- GET state changers (`/api/logout`, `/api/denormalize/...`) are not
  protected.
- An XSS anywhere on `*.scmscx.com` defeats the cookie entirely.

**Fix:** Add a double-submit CSRF token, or check `Origin` / `Referer` on
state-changing requests against an allowlist of own origins.

### 14. Tracking-analytics fingerprint uses a hardcoded salt

`crates/bwmapserver/src/middleware/trackinganalytics.rs:58-103`. The "salt"
is a fixed string baked into the binary; inputs are low-entropy (UA + IP +
accept-language + sec-ch-ua\*). Anyone reading this source can recompute
`tac` for a target IP/UA, which links log entries to a person. The cookie
is set with `http_only(false)` so JS can read it.

`tac` is currently only used for logging, so the impact is privacy / log
de-anonymization rather than authn/authz. If `tac` ever gates access in the
future, this becomes more serious.

**Fix:** Move the salt to env config (`TAC_SALT`), rotate it periodically,
and document `tac` as a non-secret correlation ID. Don't tie it to access
decisions.

---

## Low / informational

- **L1 — No password complexity / length minimum.** `api/register.rs:37-47`
  accepts a 1-character password. Combined with #1/#4, brute force is
  trivial for users who pick simple passwords.
- **L2 — Hash input includes username.** `db.rs:77, 112, 145, 253, 285` mix
  the username into the hash. `change_username` rebuilds the hash because of
  this, but it means renaming a user requires the password to be supplied
  (which it is — fine), and concurrent password/username changes can desync.
  Tiny race; mostly cosmetic.
- **L3 — `register` returns `Unauthorized` with "Could not register account"**
  when the username already exists (`api/register.rs:76`). Leaks user
  existence via both status code and message. Use 409 Conflict with a
  generic message.
- **L4 — `/api/get_selection_of_random_maps`** has admin gating in the
  handler but its SQL has no LIMIT and selects every non-NSFW map. Slow DB
  query DoS for the admin endpoints.
- **L5 — View / download counters are pumpable.** `/api/uiv2/map_info/{id}`
  unconditionally `update map set views=views+1` (`map_info.rs:386-396`),
  and `/api/maps/{mapblob_hash}` (`actix.rs:103-119`) increments `downloads`
  unless the `User-Agent` contains the literal string `"norecord"`. Trivial
  to game.
- **L6 — Mid-stream write failure leaks temp files.** `actix.rs:178-201`
  mirrors every served `b2_download_file_by_name` chunk to
  `./pending/downloading/{uuid}.scx` and removes the file when the stream
  finishes. If a write fails mid-stream (`temp.write_all(&chunk)` errors),
  the code drops the file handle but the on-disk file is left behind — the
  `tokio::fs::remove_file` call only runs after the streaming loop completes
  successfully. Repeated disk-full / I/O errors leak partials.
- **L7 — `Files::new("/", "./public/")`** is registered last
  (`actix.rs:1058-1064`) with `disable_content_disposition()` and
  `prefer_utf8(true)`. Verify that the deployed `./public/` doesn't contain
  dotfiles, `.git`, backups, or anything else sensitive.
- **L8 — `serde_json::to_string(...).unwrap()`** in several handlers
  (`actix.rs:349, 544, 599`, `chk.rs:54, 94, 132, 172, 212, 256`). These
  can't realistically panic on internally-constructed values, but they're
  panic surfaces if those types ever change.

---

## Recommended remediation order

1. Migrate password hashing to Argon2id; force re-hash on next login. (#1)
2. Hash `account.token` at rest, stop logging cookies and tokens, rotate the
   token on `change_password` / `change_username`. (#2, #3, #11)
3. Require current password on `/api/change-password`. (#3)
4. Per-IP and per-username rate limit on `/api/login`, `/api/register`, and
   `/api/uiv2/upload-map`; hard cap on upload size + cleanup on every error
   path. (#4, #5)
5. Close the remaining authorization holes: NSFW / blackholed gating on
   `/api/replays/*`, `/api/chk/*`, and `/api/search_result_popup/*`. (#7;
   `add_tags`, `/api/minimap/{chk_id}`, `/api/minimap_resized/*`, the flags
   POST status code, and `/api/tests/all_maps` are already fixed.)
6. Constant-time token compare in `UserSessionTransformer`. (#9; #10 already
   mitigated by removing the redundant handler-level auth checks.)
7. Move admin out of hardcoded ID literals into a DB role. (#7)
8. Compile the dev proxy out of release builds (or restrict its allowed
   paths). (#6)
9. Address #8, #12, #13, #14, then the L-series cleanups.

## Out of scope for this audit

- Dependency CVEs. Run `cargo audit` and `cargo deny check advisories` and
  tie them into CI.
- Binary parser hardening. Uploads pass attacker-controlled MPQ/CHK bytes
  into Rust + FFI parsers (`stormlib-bindings`, `chkdraft-bindings`,
  `bwmpq`, `bwmap`). Worth a fuzzing pass with `cargo fuzz` or AFL.
- Infrastructure: TLS termination, DB user privileges, backup encryption,
  reverse-proxy rate limiting.
- Frontend (`app/`) XSS sinks. SSR templates use `{{ }}` (auto-escaped) and
  no triple-stash, so the SSR side is safe; the React app was not reviewed.
