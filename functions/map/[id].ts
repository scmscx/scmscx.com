import { AppendHtml, Env, fetchShell, SetAttr, SetText } from "../_shared";
import { originFetch } from "../_origin";

interface MapMeta {
  web_id: string;
  scenario: string;
  scenario_description: string;
  nsfw: boolean;
  blackholed: boolean;
}

// GET /map/:id — inject per-map OpenGraph tags into the SPA shell.
export const onRequestGet: PagesFunction<Env> = async (ctx) => {
  const { request, env, params } = ctx;
  const requested = String(params.id);
  const origin = new URL(request.url).origin;

  // originFetch returns a Response even when the origin is unreachable (a 502), so
  // meta just stays null on failure; only the JSON parse can throw.
  let meta: MapMeta | null = null;
  const res = await originFetch(
    env,
    `/api/uiv2/map_meta/${encodeURIComponent(requested)}`
  );
  if (res.ok) {
    try {
      meta = await res.json<MapMeta>();
    } catch {
      // Malformed meta body -> leave meta null -> generic shell below.
    }
  }

  // Unknown or hidden (nsfw / blackholed) map: serve a generic preview, and do NOT
  // redirect first — a 301 on a hidden map would confirm it exists and leak its
  // canonical web id. Previews are always unauthenticated; the SPA + API still
  // enforce real access on the map data client-side.
  if (!meta || meta.nsfw || meta.blackholed) {
    return fetchShell(env, origin);
  }

  // Canonicalize the URL (numeric id or non-canonical web id -> canonical web id).
  if (meta.web_id && meta.web_id !== requested) {
    return Response.redirect(
      new URL(`/map/${meta.web_id}`, origin).toString(),
      301
    );
  }

  const title = `${meta.scenario} - scmscx.com`;
  const base = `Starcraft: Brood War map details and information. ${meta.scenario}`;
  const description = meta.scenario_description
    ? `${base}: ${meta.scenario_description}`
    : base;
  const ogImage = new URL(`/api/uiv2/minimap/${meta.web_id}`, origin).toString();

  return new HTMLRewriter()
    .on("title", new SetText(title))
    .on('meta[name="description"]', new SetAttr("content", description))
    .on('meta[property="og:title"]', new SetAttr("content", meta.scenario))
    .on(
      'meta[property="og:description"]',
      new SetAttr("content", meta.scenario_description)
    )
    // The shell carries no og:image (it is map-specific); append one. web_id and
    // origin are not user-controlled, so this constructed markup is safe.
    .on("head", new AppendHtml(`<meta property="og:image" content="${ogImage}">`))
    .transform(await fetchShell(env, origin));
};
