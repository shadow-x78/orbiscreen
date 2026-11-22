// ─────────────────────────────────────────────
// Orbiscreen - site worker
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────
// The site is static: every page is built ahead of time and served from the
// assets binding. Three things need code, because a path-based configuration
// file cannot see the hostname or the Accept header:
//
//  1. www answers with one 301 to the apex, preserving path and query, so the
//     domain has a single address for search engines.
//  2. Any other host - the workers.dev endpoint and every preview URL - serves
//     the same pages with X-Robots-Tag: noindex. Without it, an indexed preview
//     deployment competes with orbiscreen.com for the same content.
//  3. A request that asks for text/markdown receives the page's Markdown mirror
//     instead of the HTML. Answer engines and agents that prefer plain text get
//     the same content without parsing a layout, and every HTML response
//     advertises the mirror with a Link header so a client that did not ask can
//     still find it.

const CANONICAL_HOST = 'orbiscreen.com';
const CANONICAL_HOSTS = new Set([CANONICAL_HOST, `www.${CANONICAL_HOST}`]);
const REDIRECT_HOSTS = new Set([`www.${CANONICAL_HOST}`]);

function mirrorPath(pathname) {
  if (pathname === '/' || pathname === '') return '/index.md';
  if (pathname === '/ar' || pathname === '/ar/') return '/ar/index.md';
  return `${pathname.replace(/\/$/, '')}.md`;
}

function isDocumentPath(pathname) {
  if (pathname.endsWith('.md')) return false;
  const last = pathname.split('/').pop() ?? '';
  return last === '' || !last.includes('.');
}

function withMarkdownHeaders(response, mirrorUrl, status, body) {
  const headers = new Headers(response.headers);
  headers.set('content-type', 'text/markdown; charset=utf-8');
  headers.set('vary', 'accept');
  headers.set('content-location', mirrorUrl.pathname);
  headers.set('x-markdown', '1');
  return new Response(body, { status, headers });
}

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

    const wantsMarkdown = /\btext\/markdown\b/i.test(request.headers.get('accept') || '');
    if (wantsMarkdown && isDocumentPath(url.pathname)) {
      const mirrorUrl = new URL(request.url);
      mirrorUrl.pathname = mirrorPath(url.pathname);
      mirrorUrl.search = '';
      const mirror = await env.ASSETS.fetch(
        new Request(mirrorUrl.toString(), { method: 'GET', headers: request.headers }),
      );
      if (mirror.ok) {
        const body = request.method === 'HEAD' ? null : mirror.body;
        return withMarkdownHeaders(mirror, mirrorUrl, mirror.status, body);
      }
    }

    const response = await env.ASSETS.fetch(request);

    if (!CANONICAL_HOSTS.has(hostname)) {
      const headers = new Headers(response.headers);
      headers.set('x-robots-tag', 'noindex, nofollow');
      return new Response(response.body, { status: response.status, headers });
    }

    const contentType = response.headers.get('content-type') || '';
    if (!contentType.includes('text/html') || !isDocumentPath(url.pathname)) {
      return response;
    }

    const mirrorUrl = new URL(request.url);
    mirrorUrl.pathname = mirrorPath(url.pathname);
    mirrorUrl.search = '';
    const headers = new Headers(response.headers);
    headers.set('vary', 'accept');
    headers.append('link', `<${mirrorUrl.toString()}>; rel="alternate"; type="text/markdown"`);
    return new Response(response.body, { status: response.status, headers });
  },
};
