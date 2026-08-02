import { JSX } from "solid-js";
import { useLang } from "./context";
import table from "../../i18n/strings.json";

/**
 * Every user-visible string, keyed by a stable identifier rather than by its
 * English text.
 *
 * Keying on English reads nicely but means editing an English string silently
 * orphans every translation of it -- nothing errors, the page just reverts to
 * English in seven languages. Keys are opaque instead, so English is simply
 * another entry and rewording it touches nothing else.
 *
 * All languages for a key live together, which gives a translator the English
 * source as context right beside the line they are editing. (Note this is the
 * inverse of i18next's file-per-language layout.)
 *
 * Every key MUST carry all 8 languages: `make i18n` fails on a gap, and
 * there is deliberately no fall back to English at runtime -- a silent fallback
 * is exactly the failure this structure exists to prevent. A key that somehow
 * goes missing renders as the key itself, which is visibly wrong rather than
 * quietly wrong.
 *
 * The `about.*` entries are machine-drafted and not yet reviewed by native
 * speakers, unlike the rest, which came from the translators credited on the
 * About page. Japanese is machine-drafted throughout for the same reason: there
 * is no `about.credits.ja`, and its absence is what says so.
 */

/** What a `{name}` or `<name>` slot renders as. See `I18nText`. */
interface Slot {
  /** Renders a link. `text` is the label; without it the label comes from the
   *  string itself, via the `<name>label</name>` form. */
  href?: string;
  text?: string;
  /** Renders a <code> span, for paths and endpoint URLs. */
  code?: string;
}

interface Entry {
  /** Values for this key's slots, alongside the translations that use them. */
  slots?: Record<string, Slot>;
  /** Language code -> text. */
  [lang: string]: string | Record<string, Slot> | undefined;
}

const strings: Record<string, Entry> = table;

/** The languages the table carries. Derived, so it cannot drift from the data. */
export const SUPPORTED_LANGUAGES = Object.keys(
  strings[Object.keys(strings)[0]]
).filter((k) => k !== "slots");

/**
 * The default, and the language every entry is guaranteed to have.
 *
 * Mirrors `DEFAULT_LANG` in crates/bwmapserver/src/i18n.rs; the two sides have
 * to agree on what "no answer" resolves to, so it is named on both rather than
 * spelled `"en"` wherever it is needed.
 */
export const DEFAULT_LANGUAGE = "en";

const i18n_internal = (langcode: string, key: string): string => {
  const entry = strings[key];

  if (entry === undefined) {
    console.error(`no such translation key: ${key}`);
    return key;
  }

  // `make i18n` guarantees every key carries every language, so the fallbacks
  // below are unreachable in a checked build. They exist so that a bypassed
  // check degrades to English rather than to a blank page.
  return (entry[langcode] as string) ?? (entry.en as string) ?? key;
};

const I18nSpan = (props: { text: string }) => {
  const [lang, _] = useLang();

  return <span>{i18n_internal(lang(), props.text)}</span>;
};

// Either form of slot in an I18nText template, matched in one pass so a string
// can use both. The backreference makes `<a>x</a>` match only its own closing
// tag, so a stray `</b>` fails to match rather than swallowing the rest.
//
// Note this means literal angle brackets in copy collide with the tag form --
// `<InvalidPlayerOwner>` read as an unclosed slot until it was reworded. Prefer
// parentheses in strings; `make i18n` catches the collision either way.
const SLOT = /<([A-Za-z0-9_]+)>([\s\S]*?)<\/\1>|\{([A-Za-z0-9_]+)\}/g;

interface I18nTextProps {
  text: string;
  /**
   * Only for slots that cannot be data: an element built at the call site.
   * Everything static lives in the table's `slots` instead. The one real case is
   * the contact address, which is a component that decrypts on scroll.
   */
  values?: { [key: string]: () => JSX.Element };
}

/**
 * A translated run of text that contains inline markup, usually links.
 *
 * `I18nSpan` can only carry a flat string, so prose containing a link had to
 * either stay untranslated or be chopped into fragments -- and fragments do not
 * survive translation, because languages do not agree on word order. Instead the
 * whole sentence is one translatable string with slots in it, and a translator
 * moves the slots around freely to fit their grammar.
 *
 * There are two forms, and which to use depends on whether the text inside the
 * slot is something a translator should be touching:
 *
 *   {name}              substitutes the slot whole. Its contents are NOT
 *                       translatable -- proper nouns, file paths, endpoint URLs.
 *
 *   <name>label</name>  wraps `label` in a link. The label IS part of the
 *                       translatable string, because it is an ordinary word in
 *                       the sentence.
 *
 * What each slot renders as lives in the table beside the translations that use
 * it, not at the call site:
 *
 *   "about.faq.tech.a3": {
 *     "slots": { "chkdraft": { "href": "https://…", "text": "Chkdraft" } },
 *     "en": "… built on {chkdraft}'s renderer, which …",
 *     "ko": "… {chkdraft}의 렌더러를 …"
 *   }
 *
 * so a translator can see what a placeholder will become, and a reviewer sees
 * the sentence and its links together. It used to be split across this file and
 * the call site, which made it impossible to tell what `{chkdraft}` was without
 * opening two files, and left nothing checking that the two halves agreed.
 */
// Off-site links get the same rel every outbound link on the page carries.
// Captures nothing, so it lives out here rather than being rebuilt per render.
const anchor = (href: string, label: JSX.Element) =>
  href.startsWith("http") ? (
    <a href={href} rel="external nofollow">
      {label}
    </a>
  ) : (
    <a href={href}>{label}</a>
  );

const I18nText = (props: I18nTextProps) => {
  const [lang, _] = useLang();

  const parts = () => {
    const text = i18n_internal(lang(), props.text);
    const slots = (strings[props.text]?.slots ?? {}) as Record<string, Slot>;
    const out: (string | JSX.Element)[] = [];
    let cursor = 0;

    for (const m of text.matchAll(SLOT)) {
      out.push(text.slice(cursor, m.index));
      cursor = m.index + m[0].length;

      const [, tag, label, placeholder] = m;
      const name = placeholder ?? tag;
      const slot = slots[name];

      if (slot?.code !== undefined) {
        out.push(<code>{slot.code}</code>);
      } else if (slot?.href !== undefined) {
        // `text` for {name}; for <name>label</name> the label is in the string.
        out.push(anchor(slot.href, slot.text ?? label));
      } else if (placeholder !== undefined && props.values?.[name]) {
        out.push(props.values[name]());
      } else {
        // No slot and no call-site value. `make i18n` fails on this, so seeing it
        // in a browser means the check was bypassed; render the raw slot, which
        // is visibly wrong rather than quietly wrong.
        console.error(`i18n: ${props.text} has no slot for ${name}`);
        out.push(label ?? m[0]);
      }
    }

    out.push(text.slice(cursor));
    return out;
  };

  return <>{parts()}</>;
};

export { I18nSpan, I18nText, i18n_internal };
