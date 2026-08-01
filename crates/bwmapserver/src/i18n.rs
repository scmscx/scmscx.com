//! Server-side translations, and deciding which language a request gets.
//!
//! The string table is shared verbatim with the frontend: `i18n/strings.json` is
//! compiled in here and imported by `app/modules/language.tsx`, so there is one
//! table rather than two that drift. `make i18n` enforces that every key carries
//! every language, which is why lookups below can treat a missing language as a
//! bug rather than a routine fallback.

use std::collections::BTreeMap;

/// key -> language code -> text.
type Table = BTreeMap<String, BTreeMap<String, String>>;

const STRINGS_JSON: &str = include_str!("../../../i18n/strings.json");

/// The default, and the language every entry is guaranteed to have.
pub const DEFAULT_LANG: &str = "en";

#[derive(Debug)]
pub struct Strings {
    table: Table,
    /// Sorted, deduplicated language codes present in the table.
    languages: Vec<String>,
}

impl Strings {
    /// Parses the compiled-in table. Returns an error rather than panicking so
    /// startup fails loudly with a message instead of an unwrap backtrace.
    pub fn load() -> anyhow::Result<Self> {
        Self::parse(STRINGS_JSON)
    }

    /// The body of [`load`](Self::load), split out so the validation below can
    /// be tested against a deliberately broken table -- the compiled-in one is
    /// valid by construction, so it cannot exercise these paths.
    fn parse(json: &str) -> anyhow::Result<Self> {
        let parsed: BTreeMap<String, BTreeMap<String, serde_json::Value>> =
            serde_json::from_str(json)?;

        // Translations are strings; the `slots` that sits beside them is an
        // object. Keeping only the strings is what separates the two, and it
        // needs no field for `slots` to be discarded through. Anything else
        // dropped here reappears immediately as a missing language below.
        let table: Table = parsed
            .into_iter()
            .map(|(key, values)| {
                let texts = values
                    .into_iter()
                    .filter_map(|(lang, v)| match v {
                        serde_json::Value::String(text) => Some((lang, text)),
                        _ => None,
                    })
                    .collect();
                (key, texts)
            })
            .collect();

        let first = table
            .values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("i18n/strings.json is empty"))?;
        let languages: Vec<String> = first.keys().cloned().collect();

        anyhow::ensure!(
            languages.iter().any(|l| l == DEFAULT_LANG),
            "i18n/strings.json has no `{DEFAULT_LANG}` entries"
        );

        // The frontend's SUPPORTED_LANGUAGES is derived the same way (off the
        // first entry), so a ragged table would silently give the two sides
        // different language sets. `make i18n` also checks this; belt and braces,
        // because that check is not what runs in production.
        //
        // Checked in both directions, and by name rather than by count: a key
        // that swaps one language for another has the right number of them and
        // would otherwise pass, leaving `get` to fall back to English for a
        // language the table claims to carry. Naming the offender also beats
        // "has 8, expected 7" when someone has to go find it.
        for (key, values) in &table {
            if let Some(missing) = languages.iter().find(|l| !values.contains_key(*l)) {
                anyhow::bail!("i18n key `{key}` is missing language `{missing}`");
            }

            if let Some(extra) = values.keys().find(|l| !languages.contains(l)) {
                anyhow::bail!("i18n key `{key}` has unexpected language `{extra}`");
            }
        }

        Ok(Self { table, languages })
    }

    /// The languages the table carries, e.g. `["de", "en", "es", ...]`.
    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    pub fn is_supported(&self, lang: &str) -> bool {
        self.languages.iter().any(|l| l == lang)
    }

    /// The text for `key` in `lang`.
    ///
    /// Falls back to English and then to the key itself. Both are unreachable
    /// when `make i18n` passes; rendering the key is deliberately ugly so a
    /// mistake shows up on the page rather than hiding as plausible English.
    // `key` shares the return lifetime because it is what comes back when the
    // lookup misses.
    pub fn get<'a>(&'a self, key: &'a str, lang: &str) -> &'a str {
        let Some(values) = self.table.get(key) else {
            return key;
        };

        values
            .get(lang)
            .or_else(|| values.get(DEFAULT_LANG))
            .map_or(key, String::as_str)
    }
}

/// One entry of an `Accept-Language` header.
struct Ranged<'a> {
    tag: &'a str,
    quality: f32,
}

/// Picks the best supported language from an `Accept-Language` header.
///
/// Entries are ranked by their q-value, highest first, because the header is not
/// required to be in preference order -- unlike `navigator.languages`, which the
/// frontend can read straight through. Region subtags are dropped (`ko-KR` ->
/// `ko`) since the table is keyed by primary subtag. `q=0` means "not
/// acceptable" and is skipped; `*` is ignored, as it says nothing about which of
/// our languages to prefer.
pub fn from_accept_language(header: &str, strings: &Strings) -> Option<String> {
    let mut ranked: Vec<Ranged> = header
        .split(',')
        .filter_map(|part| {
            let mut bits = part.split(';');
            let tag = bits.next()?.trim();

            if tag.is_empty() || tag == "*" {
                return None;
            }

            let quality = bits
                .find_map(|p| p.trim().strip_prefix("q=")?.parse::<f32>().ok())
                .unwrap_or(1.0);

            (quality > 0.0).then_some(Ranged { tag, quality })
        })
        .collect();

    // Stable sort, so equal q-values keep the order the client wrote them in.
    ranked.sort_by(|a, b| {
        b.quality
            .partial_cmp(&a.quality)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ranked.into_iter().find_map(|r| {
        let primary = r.tag.split('-').next()?.to_ascii_lowercase();
        strings.is_supported(&primary).then_some(primary)
    })
}

/// Where a resolved language came from, which decides whether it gets stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The visitor already has it stored; nothing to write.
    Cookie,
    /// A language the visitor asked for and we have. Worth storing.
    AcceptLanguage,
    /// Nothing the visitor asked for matched, so this is the default.
    Default,
}

/// Decides a request's language, mirroring the frontend's order in
/// `app/modules/context.tsx`:
///
///   1. the `lang2` cookie, when it names a language we have
///   2. the best supported match from `Accept-Language`
///   3. English
///
/// The [`Source`] matters as much as the language. Only `AcceptLanguage` is
/// worth storing: writing the cookie on `Default` would freeze a visitor whose
/// language we don't carry onto English permanently, so the day we add their
/// language they would never see it. Leaving them cookie-less means every
/// request re-resolves, and they start getting their own language the moment it
/// exists.
pub fn resolve(
    cookie: Option<&str>,
    accept_language: Option<&str>,
    strings: &Strings,
) -> (String, Source) {
    // An unsupported cookie value is treated as absent: it is either stale from
    // a language we dropped, or hand-edited. It is left in place rather than
    // cleared -- it simply never matches, so every request re-resolves, which is
    // the same behavior as having no cookie at all.
    if let Some(lang) = cookie.filter(|c| strings.is_supported(c)) {
        return (lang.to_string(), Source::Cookie);
    }

    accept_language
        .and_then(|header| from_accept_language(header, strings))
        .map_or_else(
            || (DEFAULT_LANG.to_string(), Source::Default),
            |lang| (lang, Source::AcceptLanguage),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings() -> Strings {
        Strings::load().expect("the compiled-in table parses")
    }

    /// A key that swaps one language for another carries the right *number* of
    /// them, so a count-only check waves it through -- and then `get` quietly
    /// serves English for a language `languages()` says the table has.
    #[test]
    fn a_key_that_swaps_a_language_is_rejected() {
        let err = Strings::parse(
            r#"{
                "a.key": { "en": "one", "ko": "하나" },
                "b.key": { "en": "two", "ja": "二" }
            }"#,
        )
        .expect_err("a table missing `ko` on one key must not load");

        let err = err.to_string();
        assert!(err.contains("b.key"), "{err}");
        assert!(err.contains("ko"), "{err}");
    }

    #[test]
    fn a_complete_table_parses() {
        let s = Strings::parse(
            r#"{
                "a.key": { "en": "one", "ko": "하나" },
                "b.key": { "en": "two", "ko": "둘" }
            }"#,
        )
        .expect("a complete table loads");

        assert_eq!(s.languages(), ["en", "ko"]);
        assert_eq!(s.get("b.key", "ko"), "둘");
    }

    #[test]
    fn table_is_complete_and_has_the_expected_languages() {
        let s = strings();
        let mut langs = s.languages().to_vec();
        langs.sort();
        assert_eq!(langs, ["de", "en", "es", "fr", "ja", "ko", "ru", "zh"]);
    }

    #[test]
    fn lookup_returns_the_requested_language() {
        let s = strings();
        assert_eq!(s.get("about.faq.api.q", "en"), "Is there an API?");
        assert_eq!(s.get("about.faq.api.q", "ko"), "API가 있나요?");
    }

    #[test]
    fn unknown_key_renders_as_the_key() {
        assert_eq!(strings().get("no.such.key", "en"), "no.such.key");
    }

    #[test]
    fn unknown_language_falls_back_to_english() {
        let s = strings();
        assert_eq!(s.get("about.faq.api.q", "xx"), "Is there an API?");
    }

    #[test]
    fn accept_language_ranks_by_q_value_not_by_position() {
        let s = strings();
        // English is listed first but outranked.
        assert_eq!(
            from_accept_language("en;q=0.5,ko;q=0.9", &s).as_deref(),
            Some("ko")
        );
    }

    #[test]
    fn accept_language_drops_region_subtags() {
        let s = strings();
        assert_eq!(
            from_accept_language("ko-KR,ko;q=0.9,en-US;q=0.8", &s).as_deref(),
            Some("ko")
        );
    }

    #[test]
    fn accept_language_skips_languages_we_do_not_have() {
        let s = strings();
        assert_eq!(
            from_accept_language("th,vi;q=0.9,de;q=0.5", &s).as_deref(),
            Some("de")
        );
        assert_eq!(from_accept_language("th,vi", &s), None);
    }

    #[test]
    fn accept_language_ignores_wildcards_and_zero_quality() {
        let s = strings();
        assert_eq!(from_accept_language("*", &s), None);
        // q=0 means explicitly not acceptable.
        assert_eq!(
            from_accept_language("de;q=0,ko;q=0.1", &s).as_deref(),
            Some("ko")
        );
    }

    #[test]
    fn equal_quality_keeps_client_order() {
        let s = strings();
        assert_eq!(from_accept_language("fr,de", &s).as_deref(), Some("fr"));
        assert_eq!(from_accept_language("de,fr", &s).as_deref(), Some("de"));
    }

    #[test]
    fn resolve_prefers_a_valid_cookie_over_the_header() {
        let s = strings();
        assert_eq!(
            resolve(Some("de"), Some("ko"), &s),
            ("de".to_string(), Source::Cookie)
        );
    }

    #[test]
    fn resolve_ignores_an_unsupported_cookie() {
        let s = strings();
        assert_eq!(
            resolve(Some("xx"), Some("ko"), &s),
            ("ko".to_string(), Source::AcceptLanguage)
        );
    }

    #[test]
    fn resolve_falls_back_to_english() {
        let s = strings();
        assert_eq!(resolve(None, None, &s), ("en".to_string(), Source::Default));
        assert_eq!(
            resolve(None, Some("th"), &s),
            ("en".to_string(), Source::Default)
        );
    }

    /// The whole point of distinguishing `Default` from `AcceptLanguage`: a
    /// visitor asking for a language we do not carry must not have English
    /// written down, or adding their language later would never reach them.
    #[test]
    fn an_unmatched_language_is_not_worth_storing() {
        let s = strings();
        for header in ["th", "th,vi;q=0.9", "*"] {
            let (lang, source) = resolve(None, Some(header), &s);
            assert_eq!(lang, "en", "{header}");
            assert_eq!(source, Source::Default, "{header} must not be stored");
        }

        // Whereas one we do carry is.
        let (_, source) = resolve(None, Some("th,de;q=0.5"), &s);
        assert_eq!(source, Source::AcceptLanguage);
    }
}
