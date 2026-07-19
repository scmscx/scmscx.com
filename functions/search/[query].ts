import { Env, fetchShell, SetAttr, SetText } from "../_shared";
import { originFetch } from "../_origin";

interface SearchCount {
  count: number;
}

// GET /search/:query — inject the "{N} maps found for: {query}" preview title into
// the SPA shell. Bare /search (no query) stays static and is served by Pages.
export const onRequestGet: PagesFunction<Env> = async (ctx) => {
  const { request, env, params } = ctx;
  // Pages already URL-decodes the route param; decoding again would double-decode
  // and throw on a literal '%'. Re-encode when building the origin URL below.
  const query = String(params.query);
  const url = new URL(request.url);
  const origin = url.origin;

  // Preserve the user's filter params so the count matches the results page.
  // originFetch returns a Response even if the origin is unreachable, so a failure
  // just leaves count null; only the JSON parse can throw.
  let count: number | null = null;
  const res = await originFetch(
    env,
    `/api/uiv2/search_count/${encodeURIComponent(query)}${url.search}`
  );
  if (res.ok) {
    try {
      count = (await res.json<SearchCount>()).count;
    } catch {
      // Malformed count body -> leave count null -> static default title.
    }
  }

  if (count === null) return fetchShell(env, origin);

  // `query` is reflected user input; SetText / SetAttr escape it (text and attribute
  // context respectively), so it cannot inject markup.
  const title = `${count} maps found for: ${query} - scmscx.com`;

  return new HTMLRewriter()
    .on("title", new SetText(title))
    .on('meta[property="og:title"]', new SetAttr("content", title))
    .transform(await fetchShell(env, origin));
};
