// ─────────────────────────────────────────────
// Orbiscreen - site worker
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────
// The site is static: every page is built ahead of time and served from the
// assets binding. The only thing that needs code is the canonical host, because
// a path-based _redirects file cannot see the hostname. Everything on
// www.orbiscreen.com answers with a single 301 to the apex, preserving the path
// and query, so the domain has one address for search engines.

const CANONICAL_HOST = 'orbiscreen.com';
const REDIRECT_HOSTS = new Set(['www.orbiscreen.com']);

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

    return env.ASSETS.fetch(request);
  },
};
