#!/usr/bin/env node
//
// Checks i18n/strings.json against the keys the app actually uses.
//
//   node tools/check-i18n.mjs            # report, exit 1 on any problem
//   node tools/check-i18n.mjs --verbose  # also list every key per problem
//
// These are errors:
//
//   1. INCOMPLETE  a key that does not carry every language, or that carries
//                  one the rest of the table does not. There is no English
//                  fallback at runtime by design -- a silent fallback is the
//                  failure the abstract-key structure exists to prevent, so a
//                  gap has to fail here instead. The extra-language direction
//                  matters just as much: the server validates both at startup
//                  and refuses to boot on a mismatch, so a check that only
//                  looked for gaps would pass CI and then fail to deploy.
//   2. UNDEFINED   a key used in a component with no entry in language.tsx.
//                  Renders as the raw key on the page.
//   3. UNUSED      an entry no component references. Harmless at runtime, but
//                  it is how dead strings accumulate, and it is the signature
//                  of a key being renamed in one place and not the other.
//   4. PLACEHOLDER a translation whose {name} slots differ from English. Those
//                  slots are substituted with real elements (usually links) at
//                  render time, so dropping one silently deletes the link from
//                  the sentence and inventing one renders literal braces.
//   5. RAW IN SSR  a key used from a .hbs template that contains placeholders.
//                  The server's {{t}} helper writes the string as-is -- it has
//                  no values to substitute -- so the braces would reach the page.
//   6. TAGS        a <name>label</name> link slot that is unbalanced, nested, or
//                  whose set differs across languages -- an unclosed tag eats the
//                  rest of the sentence -- or that no call site supplies a link
//                  for, which would silently downgrade the link to plain text.
//
// Keys are read out of the components syntactically, from <I18nSpan text="...">,
// <I18nText text="...">, i18n_internal(..., "..."), and any `key`-shaped string
// literal (dotted lowercase), which covers keys handed to a component as a prop
// -- `<MapBox h4="home.featured_maps">` reaches I18nSpan indirectly.

import { readdirSync, readFileSync, statSync } from "fs";
import { join } from "path";

const APP_DIR = "app";
const TEMPLATE_DIR = join("app", "web", "uiv2");
// Some keys are resolved in the server rather than a component or a template --
// anything whose text has to be assembled before rendering.
const RUST_DIR = join("crates", "bwmapserver", "src");
// The single source of truth, compiled into the server and imported by the
// frontend. Neither side declares its own copy.
const STRINGS = join("i18n", "strings.json");
const IMPL = join("app", "modules", "language.tsx");
// Anything shaped like `namespace.some_key`.
const KEY_SHAPE = /^[a-z][a-z0-9]*(?:\.[a-z0-9_]+)+$/;

function walk(dir) {
  return readdirSync(dir).flatMap((name) => {
    const p = join(dir, name);
    return statSync(p).isDirectory() ? walk(p) : [p];
  });
}

/** Key-shaped string literals under `dir`, with the files that reference them.
 *
 * Serves both sides: the components (`.tsx`/`.ts`, skipping the implementation
 * itself) and the server (`.rs`), which looks keys up directly, e.g.
 * strings.get("meta.search.title", lang). One scanner rather than two, so the
 * literal regex and the key-shape gate cannot drift apart.
 *
 * `namespaces` gates the match: a key-shaped literal is only a key if its
 * namespace is one the table actually declares. Without that, ordinary text
 * like a bare domain ("staredit.net") reads as a key and reports as undefined.
 *
 * Comments are stripped on both sides: a key that only survives inside
 * commented-out JSX -- or inside a Rust doc comment -- is not live, and should
 * not oblige anyone to translate it.
 */
function scanLiterals(dir, exts, skip) {
  const used = new Map();
  // .ts as well as .tsx: helpers in app/util/ return keys that components feed
  // straight to I18nSpan, and scanning only .tsx made those keys look unused --
  // which is how a batch of them got deleted as "orphans" while still reachable.
  for (const file of walk(dir).filter((f) => exts.some((e) => f.endsWith(e)))) {
    if (file === skip) continue;
    const src = readFileSync(file, "utf8")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/^\s*\/\/.*$/gm, "");

    for (const m of src.matchAll(/"((?:[^"\\\n]|\\.)*)"/g)) {
      if (!KEY_SHAPE.test(m[1])) continue;
      if (!namespaces.has(m[1].split(".")[0])) continue;
      if (!used.has(m[1])) used.set(m[1], new Set());
      used.get(m[1]).add(file);
    }
  }
  return used;
}

/** Keys the server-rendered handlebars templates ask for via {{t "..."}}. */
function templateKeys() {
  const used = new Map();
  for (const file of walk(TEMPLATE_DIR).filter((f) => f.endsWith(".hbs"))) {
    const src = readFileSync(file, "utf8");
    for (const m of src.matchAll(/\{\{\s*t\s+"([^"]+)"/g)) {
      if (!used.has(m[1])) used.set(m[1], new Set());
      used.get(m[1]).add(file);
    }
  }
  return used;
}

const verbose = process.argv.includes("--verbose");

const strings = JSON.parse(readFileSync(STRINGS, "utf8"));
if (Object.keys(strings).length === 0) throw new Error(`${STRINGS} is empty`);

// `slots` sits beside the languages; it is data for the placeholders, not a
// translation.
const langsOf = (entry) => Object.keys(entry).filter((k) => k !== "slots");
/** The declared table: key -> set of languages present. */
const table = new Map(Object.entries(strings).map(([k, v]) => [k, new Set(langsOf(v))]));
// Derived from the data, like the frontend and the server both do.
const LANGS = [...table.values().next().value].sort();
const namespaces = new Set([...table.keys()].map((k) => k.split(".")[0]));

const templates = templateKeys();
const used = scanLiterals(APP_DIR, [".tsx", ".ts"], IMPL);
for (const source of [templates, scanLiterals(RUST_DIR, [".rs"])]) {
  for (const [key, files] of source) {
    if (!used.has(key)) used.set(key, new Set());
    files.forEach((f) => used.get(key).add(f));
  }
}

console.log(`${table.size} keys declared, ${used.size} used across the components\n`);

const incomplete = [];
const extraLanguages = [];
for (const [key, langs] of table) {
  const missing = LANGS.filter((l) => !langs.has(l));
  if (missing.length) incomplete.push([key, missing]);
  // The other direction matters just as much. `Strings::parse` rejects a key
  // carrying a language the rest of the table does not, so a check that only
  // looked for gaps would pass here and then refuse to boot in production.
  const extra = [...langs].filter((l) => !LANGS.includes(l)).sort();
  if (extra.length) extraLanguages.push([key, extra]);
}
// Placeholder slots must match English exactly; order is irrelevant, since a
// translation is expected to move them to suit its grammar.
const slots = (s) => [...new Set(s.match(/\{[A-Za-z0-9_]+\}/g) ?? [])].sort().join(",");
// Balanced <name>...</name> pairs. Anything left over after stripping them is
// malformed: an unclosed tag, a stray close, or nesting.
const TAG_PAIR = /<([A-Za-z0-9_]+)>((?:(?!<\/?[A-Za-z0-9_]+>)[\s\S])*?)<\/\1>/g;
const tags = (s) => [...new Set([...s.matchAll(TAG_PAIR)].map((m) => m[1]))].sort();
const strayTags = (s) => s.replace(TAG_PAIR, "").match(/<\/?[A-Za-z0-9_]+>/g) ?? [];

// Every slot a string references must have a value in the same entry, except
// these, which are filled in at runtime and so cannot be data. Listed per key
// rather than by name, so the exemption stays narrow and each one is justified.
const RUNTIME_SLOTS = new Set([
  // A component that decrypts the address on scroll -- see modules/Email.tsx.
  "about.faq.contact.a.email",
  "about.faq.removal.a.email",
  // Substituted in the server, which knows the result count and the query.
  "meta.search.results.count",
  "meta.search.results.query",
]);

// One pass over the table: the placeholder, link-tag and slot checks are
// independent, and none reads another's results.
const placeholderMismatches = [];
const malformedTags = [];
const tagMismatches = [];
const missingSlots = [];
const unusedSlots = [];
const decomposed = [];
for (const [key, values] of Object.entries(strings)) {
  const wantSlots = slots(values.en ?? "");
  const wantTags = tags(values.en ?? "").join(",");
  const declaredSlots = values.slots ?? {};
  const referenced = new Set();

  for (const lang of langsOf(values)) {
    const text = values[lang];
    if (slots(text) !== wantSlots) {
      placeholderMismatches.push([key, lang, slots(text), wantSlots]);
    }
    const stray = strayTags(text);
    if (stray.length) malformedTags.push([key, lang, stray.join(" ")]);
    if (tags(text).join(",") !== wantTags) {
      tagMismatches.push([key, lang, tags(text).join(","), wantTags]);
    }
    // A decomposed accent (o + U+0301 rather than a single ó) is invisible in
    // an editor and wrong on the page: the display font covers the precomposed
    // letter but not the combining mark, so the base letter and its accent get
    // drawn by two different fonts and the accent lands badly. Found exactly
    // this in the Spanish "Razón".
    if (text !== text.normalize("NFC")) decomposed.push([key, lang]);
    for (const m of text.matchAll(/\{([A-Za-z0-9_]+)\}/g)) referenced.add(m[1]);
    for (const m of text.matchAll(TAG_PAIR)) referenced.add(m[1]);
  }

  for (const name of referenced) {
    if (declaredSlots[name] === undefined && !RUNTIME_SLOTS.has(`${key}.${name}`)) {
      missingSlots.push([key, name]);
    }
  }
  for (const name of Object.keys(declaredSlots)) {
    if (!referenced.has(name)) unusedSlots.push([key, name]);
  }
}

const rawInTemplates = [...templates.keys()].filter(
  (k) => strings[k] !== undefined && slots(strings[k].en) !== ""
);

const undefinedKeys = [...used.keys()].filter((k) => !table.has(k));
const unusedKeys = [...table.keys()].filter((k) => !used.has(k));

for (const lang of LANGS) {
  const have = [...table.values()].filter((s) => s.has(lang)).length;
  console.log(`  ${lang}: ${have}/${table.size}${have === table.size ? "" : "  <-- INCOMPLETE"}`);
}

let failed = false;
const report = (label, items, render, { stream = console.error, limit = 15, fatal = true } = {}) => {
  if (items.length === 0) return;
  if (fatal) failed = true;
  stream(`\n${items.length} ${label}:`);
  for (const it of verbose ? items : items.slice(0, limit)) stream("  " + render(it));
  if (!verbose && items.length > limit) {
    stream(`  ... and ${items.length - limit} more (--verbose)`);
  }
};

report("key(s) missing a language", incomplete, ([k, m]) => `${k}  missing: ${m.join(",")}`);
report(
  "key(s) carrying a language the rest of the table does not",
  extraLanguages,
  ([k, e]) => `${k}  unexpected: ${e.join(",")}  (the server refuses to start on this)`
);
report("key(s) used but not declared", undefinedKeys, (k) => `${k}  (${[...used.get(k)].join(", ")})`);
report("declared key(s) never used", unusedKeys, (k) => k);
report(
  "translation(s) whose placeholders differ from English",
  placeholderMismatches,
  ([k, l, got, want]) => `${k} [${l}]  has: ${got || "(none)"}  expected: ${want || "(none)"}`
);
report(
  "malformed link tag(s)",
  malformedTags,
  ([k, l, stray]) => `${k} [${l}]  unbalanced or nested: ${stray}`
);
report(
  "translation(s) whose link tags differ from English",
  tagMismatches,
  ([k, l, got, want]) => `${k} [${l}]  has: ${got || "(none)"}  expected: ${want || "(none)"}`
);
report(
  "slot(s) referenced with no value",
  missingSlots,
  ([k, name]) => `${k}  ${name} has no entry in its "slots" object`
);
report(
  "declared slot(s) no translation references",
  unusedSlots,
  ([k, name]) => `${k}  "slots".${name} is never used`
);
report(
  "translation(s) holding a decomposed accent",
  decomposed,
  ([k, l]) => `${k} [${l}]  is not NFC -- the accent renders from a fallback font`
);
report(
  "key(s) used from a template but containing placeholders",
  rawInTemplates,
  (k) => `${k}  ${slots(strings[k].en)} would render literally -- {{t}} does not interpolate`
);

if (failed) {
  console.error(
    `\nEvery key must carry all ${LANGS.length} languages and be used exactly where it is\n` +
      `declared. There is no English fallback at runtime.`
  );
  process.exit(1);
}
// Not an error: a key whose every language equals English is a placeholder
// awaiting a native speaker, which is deliberate for terms where we do not yet
// know what the community actually says. Reported so the backlog is visible
// rather than indistinguishable from a finished translation.
const untranslated = [...table.keys()].filter((k) => {
  const v = strings[k];
  return LANGS.every((l) => v[l] === v.en);
});
report(
  "key(s) are English in every language, awaiting a translator",
  untranslated,
  (k) => k,
  { stream: console.log, limit: 10, fatal: false }
);

console.log("\nall keys complete in every language, and all are used");
