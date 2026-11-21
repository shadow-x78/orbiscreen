// ─────────────────────────────────────────────
// Orbiscreen - site worker
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────
// The site is static: every page is built ahead of time and served from the
// assets binding. Two things need code, because a path-based _redirects file
// cannot see the hostname:
//
//  1. www answers with one 301 to the apex, preserving path and query, so the
//     domain has a single address for search engines.
//  2. Any other host - the workers.dev endpoint and every preview URL - serves
//     the same pages with X-Robots-Tag: noindex. Without it, an indexed preview
//     deployment competes with orbiscreen.com for the same content.

const CANONICAL_HOST = 'orbiscreen.com';
const CANONICAL_HOSTS = new Set([CANONICAL_HOST, `www.${CANONICAL_HOST}`]);
const REDIRECT_HOSTS = new Set([`www.${CANONICAL_HOST}`]);

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    // The Host header is the hostname the visitor asked for. On Cloudflare it
    // always matches the URL; `wrangler dev` rewrites the URL to localhost and
    // maps routes itself, so the header is the reliable source in both.
    const headerHost = request.headers.get('host');
    const hostname = (headerHost ? headerHost.split(':')[0] : url.hostname).toLowerCase();

    if (REDIRECT_HOSTS.has(hostname)) {
      url.protocol = 'https:';
      url.hostname = CANONICAL_HOST;
      url.port = '';
      return new Response(null, {
        status: 301,
        headers: {
          location: url.toString(),
          'cache-control': 'public, max-age=3600',
        },
      });
    }

    const response = await env.ASSETS.fetch(request);
    if (CANONICAL_HOSTS.has(hostname)) return response;

    const headers = new Headers(response.headers);
    headers.set('x-robots-tag', 'noindex, nofollow');
    return new Response(response.body, { status: response.status, headers });
  },
};
