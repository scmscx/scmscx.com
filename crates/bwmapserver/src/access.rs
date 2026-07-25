//! Who is allowed to see or change what.
//!
//! These rules were previously copy-pasted across handlers — the admin account id
//! alone appeared in eight files in five different spellings — which made them
//! impossible to audit and easy to get subtly wrong. A missed check leaks a
//! blackholed or NSFW map, so they live in one place now.
//!
//! Privilege comes from `account.role` rather than a compiled-in account id, so
//! it can be granted and revoked without a deploy, and so tests can exercise the
//! privileged paths at all. The role is read with the session in the
//! `user_session` middleware and rides on [`UserSession`]; nothing here queries
//! the database for it.
//!
//! Deliberately only *predicates*: callers keep their own control flow and decide
//! which status code to answer with. That is not an oversight. The endpoints
//! genuinely disagree — most check NSFW first and answer `403`, while
//! `search_result_popup` checks blackholed first and answers `401` with cache
//! headers — so folding the decision in here would silently change responses.

use crate::middleware::UserSession;
use crate::webutil::{Pool, PoolExt};

/// A privilege level, stored in `account.role`.
///
/// Roles are hierarchical — an admin can do anything a moderator can — which is
/// the derived `Ord`, so gates are written `role >= Role::Moderator`. **The
/// variants are declared least privileged first and reordering them silently
/// rewrites every gate in this file.**
///
/// [`Role::User`] is the ordinary account, so every session has a role and none
/// of this deals in `Option<Role>`. It has no spelling in the database: the
/// column only ever names a privilege grant, and the ordinary case is `NULL`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    User,
    Moderator,
    Admin,
}

impl Role {
    /// Resolve an `account.role` value. `NULL` is the ordinary account, and a
    /// value this build doesn't recognise resolves the same way rather than being
    /// fatal — a role written ahead of the code that understands it must not
    /// break every request for that account, and must not grant anything either.
    ///
    /// The column stores only privilege grants, so there is no `'user'` case to
    /// match: it arrives as `NULL` and falls through with everything else.
    pub fn from_db(value: Option<&str>) -> Self {
        match value {
            Some("admin") => Self::Admin,
            Some("moderator") => Self::Moderator,
            _ => Self::User,
        }
    }
}

/// Whether the viewer is an admin. Anonymous never is.
pub fn is_admin(user: Option<&UserSession>) -> bool {
    user.is_some_and(|u| u.role >= Role::Admin)
}

/// Whether the viewer may use the moderation tooling. True for moderators and,
/// because roles are hierarchical, for admins too.
pub fn can_moderate(user: Option<&UserSession>) -> bool {
    user.is_some_and(|u| u.role >= Role::Moderator)
}

/// Whether a logged-in user may edit a map: its uploader, or an admin.
pub fn may_modify_map(uploaded_by: i64, user: Option<&UserSession>) -> bool {
    user.is_some_and(|u| u.id == uploaded_by) || is_admin(user)
}

/// Whether a blackholed map must be hidden from this viewer.
///
/// Blackholing is a moderation action and is meant to be indistinguishable from
/// the map having been deleted: it hides the map from *everyone* except an admin,
/// including its uploader. Endpoints answer `404` — never `403` — so the map's
/// continued existence isn't confirmed either.
pub fn blackholed_is_hidden_from(blackholed: bool, user: Option<&UserSession>) -> bool {
    blackholed && !is_admin(user)
}

/// Whether an NSFW map is gated for this viewer. Any logged-in account may see
/// NSFW content; anonymous visitors may not.
pub fn nsfw_requires_login(nsfw: bool, user: Option<&UserSession>) -> bool {
    nsfw && user.is_none()
}

/// Outcome of [`check_chk_access`].
pub enum ChkAccess {
    /// Access granted. `restricted` is true when any map referencing this chk is
    /// NSFW or blackholed — i.e. the response was only served because the caller
    /// is authorized. Callers use it to keep such content out of shared caches.
    Allowed {
        restricted: bool,
    },
    NotFound,
    Unauthorized,
}

impl ChkAccess {
    /// The `restricted` flag when access is granted, or the status to refuse
    /// with when it isn't.
    ///
    /// Unlike the per-map gates, every chk caller refuses identically — `404`
    /// for missing-or-blackholed, `401` for NSFW-without-a-login — and differs
    /// only in the headers it hangs on the response. So the status mapping lives
    /// here while building the response stays with the caller, which is what
    /// lets `get_map_img` attach `no-cache` and `download_chk` fall through to
    /// the default `no-store`.
    pub fn restricted_or_refusal(self) -> Result<bool, http::StatusCode> {
        match self {
            Self::Allowed { restricted } => Ok(restricted),
            Self::NotFound => Err(http::StatusCode::NOT_FOUND),
            Self::Unauthorized => Err(http::StatusCode::UNAUTHORIZED),
        }
    }
}

/// Visibility of a chk, which is reachable through any map that references it.
///
/// A chk has no uploader of its own, so it inherits the most restrictive flag of
/// any referencing map: one blackholed map hides the whole chk, one NSFW map
/// makes the whole chk NSFW.
pub async fn check_chk_access(
    pool: &Pool,
    chk_id: &str,
    user: Option<&UserSession>,
) -> Result<ChkAccess, anyhow::Error> {
    let row = pool
        .acquire()
        .await?
        .query_one(
            "select
                count(*) > 0 as exists_any,
                coalesce(bool_or(blackholed), false) as any_blackholed,
                coalesce(bool_or(nsfw), false) as any_nsfw
             from map
             where chkblob = $1",
            &[&chk_id],
        )
        .await?;

    let exists_any: bool = row.try_get("exists_any")?;
    let any_blackholed: bool = row.try_get("any_blackholed")?;
    let any_nsfw: bool = row.try_get("any_nsfw")?;

    if !exists_any {
        return Ok(ChkAccess::NotFound);
    }
    if blackholed_is_hidden_from(any_blackholed, user) {
        return Ok(ChkAccess::NotFound);
    }
    if nsfw_requires_login(any_nsfw, user) {
        return Ok(ChkAccess::Unauthorized);
    }

    Ok(ChkAccess::Allowed {
        restricted: any_nsfw || any_blackholed,
    })
}

/// Whether a map is hidden from this viewer by the blackhole gate, looked up by
/// map id.
///
/// A map id that doesn't exist is reported as hidden, so callers answer `404`
/// without a separate existence check.
///
/// Costs a connection checkout and a round trip, so prefer folding `blackholed`
/// into a query the handler already runs — as `flags::get_flag`, `uiv2::units`
/// and `chk::visible_chkblob` do. That only works when the handler reads a
/// *single* `map` row, though. A handler that returns a list joined off another
/// table (tags, filenames, replays, timestamps) can't: its query yields zero
/// rows for a map with no tags and zero rows for a map that doesn't exist, and
/// this separate lookup is exactly what tells those apart. Reach for it there.
pub async fn map_is_hidden(
    pool: &Pool,
    map_id: i64,
    user: Option<&UserSession>,
) -> Result<bool, anyhow::Error> {
    let row = pool
        .acquire()
        .await?
        .query_opt("select blackholed from map where id = $1", &[&map_id])
        .await?;

    Ok(match row {
        None => true,
        Some(row) => blackholed_is_hidden_from(row.try_get("blackholed")?, user),
    })
}

/// Whether the map blob behind `mapblob_hash` is hidden by the blackhole gate.
///
/// A blob can be referenced by more than one map, so the most restrictive flag
/// wins, matching [`check_chk_access`]. An unknown hash is reported as hidden so
/// callers answer `404` without a separate existence check.
pub async fn mapblob_is_hidden(
    pool: &Pool,
    mapblob_hash: &str,
    user: Option<&UserSession>,
) -> Result<bool, anyhow::Error> {
    let row = pool
        .acquire()
        .await?
        .query_one(
            "select
                count(*) > 0 as exists_any,
                coalesce(bool_or(blackholed), false) as any_blackholed
             from map
             where mapblob2 = $1",
            &[&mapblob_hash],
        )
        .await?;

    if !row.try_get::<_, bool>("exists_any")? {
        return Ok(true);
    }
    Ok(blackholed_is_hidden_from(
        row.try_get("any_blackholed")?,
        user,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const UPLOADER: i64 = 77;
    const OTHER: i64 = 99;

    fn session(id: i64, role: Role) -> UserSession {
        UserSession {
            id,
            username: format!("user{id}"),
            token: "t".to_owned(),
            role,
        }
    }

    #[test]
    fn unrecognised_and_null_roles_resolve_to_the_lowest() {
        assert_eq!(Role::from_db(Some("admin")), Role::Admin);
        assert_eq!(Role::from_db(Some("moderator")), Role::Moderator);
        // NULL is the ordinary account, and anything unknown must fall to the
        // least privilege rather than to whatever the match happens to reach.
        assert_eq!(Role::from_db(None), Role::User);
        // The column stores only grants, so 'user' is not a value it can hold —
        // but a hand-written row still has to land on the ordinary account.
        assert_eq!(Role::from_db(Some("user")), Role::User);
        assert_eq!(
            Role::from_db(Some("Admin")),
            Role::User,
            "matching is exact"
        );
        assert_eq!(Role::from_db(Some("wheelbarrow")), Role::User);
    }

    #[test]
    fn the_variants_are_ordered_least_privileged_first() {
        // The gates are `>=` against the derived Ord, so this declaration order
        // *is* the privilege hierarchy. Reordering the enum would rewrite every
        // gate in this file without touching a single one of them.
        assert!(Role::User < Role::Moderator);
        assert!(Role::Moderator < Role::Admin);
    }

    #[test]
    fn a_plain_user_role_grants_nothing_privileged() {
        let u = session(OTHER, Role::User);
        assert!(!can_moderate(Some(&u)));
        assert!(!is_admin(Some(&u)));
    }

    #[test]
    fn admin_comes_from_the_role_not_the_account_id() {
        let admin = session(OTHER, Role::Admin);
        let plain = session(OTHER, Role::User);
        // Account id 4 used to be admin by fiat; it no longer is without a grant.
        let legacy = session(4, Role::User);
        assert!(is_admin(Some(&admin)));
        assert!(!is_admin(Some(&plain)));
        assert!(!is_admin(Some(&legacy)));
        assert!(!is_admin(None), "anonymous is never admin");
    }

    #[test]
    fn roles_are_hierarchical_one_way() {
        let admin = Some(session(OTHER, Role::Admin));
        let mod_ = Some(session(OTHER, Role::Moderator));
        let plain = Some(session(OTHER, Role::User));

        // An admin satisfies a moderator gate; a moderator does not satisfy an
        // admin gate. Getting this backwards would either lock admins out of the
        // tooling or hand moderators the admin-only routes.
        assert!(can_moderate(admin.as_ref()));
        assert!(can_moderate(mod_.as_ref()));
        assert!(is_admin(admin.as_ref()));
        assert!(!is_admin(mod_.as_ref()));

        assert!(!can_moderate(plain.as_ref()));
        assert!(!can_moderate(None));
    }

    #[test]
    fn a_moderator_does_not_see_blackholed_maps() {
        // Blackholing is admin-only; moderation tooling access doesn't grant it.
        let m = session(OTHER, Role::Moderator);
        assert!(blackholed_is_hidden_from(true, Some(&m)));
    }

    #[test]
    fn only_uploader_and_admin_may_modify() {
        assert!(may_modify_map(
            UPLOADER,
            Some(&session(UPLOADER, Role::User))
        ));
        assert!(may_modify_map(UPLOADER, Some(&session(OTHER, Role::Admin))));
        assert!(!may_modify_map(UPLOADER, Some(&session(OTHER, Role::User))));
        assert!(!may_modify_map(UPLOADER, None));
    }

    #[test]
    fn blackholing_hides_from_everyone_but_the_admin() {
        assert!(blackholed_is_hidden_from(true, None));
        assert!(blackholed_is_hidden_from(
            true,
            Some(&session(OTHER, Role::User))
        ));
        // The uploader is *not* exempt: blackholing must look like deletion.
        assert!(blackholed_is_hidden_from(
            true,
            Some(&session(UPLOADER, Role::User))
        ));
        assert!(!blackholed_is_hidden_from(
            true,
            Some(&session(OTHER, Role::Admin))
        ));
        // A map that isn't blackholed is never hidden by this rule — the mutant
        // that drops the `blackholed &&` would hide every map from everyone.
        assert!(!blackholed_is_hidden_from(false, None));
        assert!(!blackholed_is_hidden_from(
            false,
            Some(&session(OTHER, Role::User))
        ));
    }

    #[test]
    fn nsfw_is_gated_only_for_anonymous_viewers() {
        assert!(nsfw_requires_login(true, None));
        // Any account will do; being the uploader or an admin isn't required.
        assert!(!nsfw_requires_login(
            true,
            Some(&session(OTHER, Role::User))
        ));
        assert!(!nsfw_requires_login(false, None));
    }
}
