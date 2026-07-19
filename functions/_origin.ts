import { Env } from "./_shared";

// Single source of truth for reaching the Rust origin. The browser only ever
// talks to the Pages domain (scmscx.com); these helpers forward server-side to
// the origin (ORIGIN_BASE), so there is no CORS and cookies stay first-party.
// Origin lock-down (Cloudflare Access service token) is applied here in one place.

// Paths the Rust origin owns. Everything else is Pages (static shell + the /map
// and /search OG Functions).
const ORIGIN_PREFIXES = ["/api/"];
const ORIGIN_EXACT = new Set([
  "/sitemap.txt",
  "/a.txt",
  "/b.txt",
  "/c.txt",
  "/site.webmanifest",
]);

/// True for a request path that must be proxied to the Rust origin.
export function isOriginPath(pathname: string): boolean {
  return (
    ORIGIN_PREFIXES.some((p) => pathname.startsWith(p)) || ORIGIN_EXACT.has(pathname)
  );
}

function withAccess(env: Env, headers: Headers): Headers {
  if (env.ORIGIN_ACCESS_CLIENT_ID && env.ORIGIN_ACCESS_CLIENT_SECRET) {
    headers.set("CF-Access-Client-Id", env.ORIGIN_ACCESS_CLIENT_ID);
    headers.set("CF-Access-Client-Secret", env.ORIGIN_ACCESS_CLIENT_SECRET);
  }
  return headers;
}

/// A clear 502 instead of a bare 1101 "Worker threw exception" when the origin
/// isn't wired up. ORIGIN_BASE is set in wrangler.toml [vars]; the ORIGIN_ACCESS_*
/// secrets in the Pages dashboard. Env changes only apply to NEW deployments.
function misconfigured(detail: string): Response {
  return new Response(
    `origin proxy misconfiguration: ${detail}\n` +
      `Set ORIGIN_BASE in wrangler.toml [vars] and (for the locked-down origin) the ` +
      `ORIGIN_ACCESS_* secrets in the Pages dashboard, then redeploy.\n`,
    { status: 502, headers: { "content-type": "text/plain; charset=utf-8" } }
  );
}

/// A 502 for a transient origin failure (the `fetch` itself rejected) — distinct
/// from `misconfigured`, which is for an unset/invalid ORIGIN_BASE.
function originError(detail: string): Response {
  return new Response(`origin request failed: ${detail}\n`, {
    status: 502,
    headers: { "content-type": "text/plain; charset=utf-8" },
  });
}

/// GET a specific origin path (used by the /map and /search OG Functions to read
/// meta/count). Bypasses the public path so it does not re-enter edge routing.
export async function originFetch(
  env: Env,
  pathAndQuery: string
): Promise<Response> {
  if (!env.ORIGIN_BASE) return misconfigured("ORIGIN_BASE is not set");
  try {
    return await fetch(new URL(pathAndQuery, env.ORIGIN_BASE).toString(), {
      headers: withAccess(env, new Headers()),
    });
  } catch (e) {
    return originError((e as Error).message);
  }
}

/// Transparently proxy an inbound request to the origin, preserving method /
/// headers / body / cookies and streaming the response back (safe for large map
/// downloads). Used by the root middleware for every origin-owned path.
export async function proxyToOrigin(
  request: Request,
  env: Env
): Promise<Response> {
  if (!env.ORIGIN_BASE) return misconfigured("ORIGIN_BASE is not set");

  const incoming = new URL(request.url);
  let target: string;
  try {
    target = new URL(
      incoming.pathname + incoming.search,
      env.ORIGIN_BASE
    ).toString();
  } catch {
    return misconfigured(
      `ORIGIN_BASE is not a valid URL: ${JSON.stringify(env.ORIGIN_BASE)}`
    );
  }

  // Don't forward the inbound Host (ppe.scmscx.com / *.pages.dev) — let fetch set
  // it from the target URL, or the subrequest can loop back to this project.
  const headers = withAccess(env, new Headers(request.headers));
  headers.delete("host");

  const hasBody = request.method !== "GET" && request.method !== "HEAD";
  try {
    return await fetch(target, {
      method: request.method,
      headers,
      body: hasBody ? request.body : undefined,
      redirect: "manual",
    });
  } catch (e) {
    return originError((e as Error).message);
  }
}
