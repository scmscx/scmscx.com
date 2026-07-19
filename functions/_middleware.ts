import { Env } from "./_shared";
import { isOriginPath, proxyToOrigin } from "./_origin";

// Root middleware — runs before every routed request (static /assets/* are
// excluded via _routes.json). It server-side-proxies every origin-owned path
// (/api/*, sitemap, site.webmanifest) to the Rust origin so the browser stays
// single-origin (no CORS, first-party cookies). Everything else falls through to
// the route Functions (/map/:id, /search/:query) and then static assets / the
// SPA fallback in _redirects.
export const onRequest: PagesFunction<Env> = async (ctx) => {
  const url = new URL(ctx.request.url);
  if (isOriginPath(url.pathname)) {
    return proxyToOrigin(ctx.request, ctx.env);
  }
  return ctx.next();
};
