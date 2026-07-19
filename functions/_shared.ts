// Shared helpers for the OpenGraph-injection Pages Functions. Files prefixed with
// `_` are not routed by Cloudflare Pages.
//
// These Functions decorate the static SPA shell (index.html) with per-map /
// per-search OG tags for link previews and crawlers, then hand the same shell to
// the browser — the SolidJS app renders the real page client-side.
//
// They call `/api/uiv2/*` on the Rust origin via `originFetch` in `_origin.ts`,
// which forwards to ORIGIN_BASE (wrangler.toml). See the migration plan and the
// phase-3 runbook in docs/.

export interface Env {
  // Static-assets binding Pages injects; used to fetch the built SPA shell.
  ASSETS: Fetcher;

  // Base URL of the Rust origin (no trailing slash), e.g. "https://scmscx.com" for
  // PPE or the Tunnel hostname at cutover. Non-secret → set in wrangler.toml [vars].
  ORIGIN_BASE: string;

  // Optional Cloudflare Access service-token credentials that lock the origin
  // hostname so only this Pages project can call it. When set, they are attached
  // to every origin request. Secrets → set in the Pages dashboard. See the runbook.
  ORIGIN_ACCESS_CLIENT_ID?: string;
  ORIGIN_ACCESS_CLIENT_SECRET?: string;
}

/// Fetch the static SPA shell to decorate or return as-is.
export function fetchShell(env: Env, origin: string): Promise<Response> {
  return env.ASSETS.fetch(new URL("/index.html", origin));
}

/// Set an attribute. The runtime escapes the value for attribute context, so this
/// is safe for reflected user input.
export class SetAttr {
  constructor(private readonly name: string, private readonly value: string) {}
  element(el: Element) {
    el.setAttribute(this.name, this.value);
  }
}

/// Replace an element's text content in text mode (the runtime escapes it), so
/// this is safe for reflected user input.
///
/// The field is `content`, NOT `text`: HTMLRewriter's `.on(selector, handlers)`
/// treats a `text` property on the handler object as a text-chunk callback and
/// throws `Incorrect type for the 'text' field ... not of type 'function'` when it
/// isn't one. `element`, `comments`, `text`, `doctype`, and `end` are all reserved
/// handler names — keep instance fields clear of them.
export class SetText {
  constructor(private readonly content: string) {}
  element(el: Element) {
    el.setInnerContent(this.content);
  }
}

/// Append raw HTML into an element (e.g. <head>). The HTML is NOT escaped — only
/// use with values you construct yourself, never reflected user input.
export class AppendHtml {
  constructor(private readonly html: string) {}
  element(el: Element) {
    el.append(this.html, { html: true });
  }
}
