import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
import { routes } from '../data/routes';
import { SITE_URL } from '../lib/site';

function escapeXml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

export const GET: APIRoute = async () => {
  // Only document pages carry a real modification date (their last commit).
  // Stamping every URL with the build date would make the sitemap "change" on
  // every deploy and teach crawlers to ignore lastmod.
  const docs = await getCollection('docs');
  const modified = new Map<string, string>();
  for (const entry of docs) {
    const prefix = entry.data.lang === 'ar' ? '/ar/docs/' : '/docs/';
    modified.set(`${prefix}${entry.data.slug}`, entry.data.updated);
  }

  const entries: string[] = [];

  for (const route of routes) {
    const en = `${SITE_URL}${route.path}`;
    const ar = `${SITE_URL}${route.arPath}`;
    const alternates = [
      `    <xhtml:link rel="alternate" hreflang="en" href="${escapeXml(en)}"/>`,
      `    <xhtml:link rel="alternate" hreflang="ar" href="${escapeXml(ar)}"/>`,
      `    <xhtml:link rel="alternate" hreflang="x-default" href="${escapeXml(en)}"/>`,
    ].join('\n');

    for (const [loc, pagePath] of [
      [en, route.path],
      [ar, route.arPath],
    ] as const) {
      const lastmod = modified.get(pagePath);
      entries.push(
        [
          '  <url>',
          `    <loc>${escapeXml(loc)}</loc>`,
          ...(lastmod ? [`    <lastmod>${lastmod}</lastmod>`] : []),
          `    <changefreq>${route.changefreq}</changefreq>`,
          `    <priority>${route.priority.toFixed(1)}</priority>`,
          alternates,
          '  </url>',
        ].join('\n'),
      );
    }
  }

  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${entries.join('\n')}
</urlset>
`;

  return new Response(xml, {
    headers: { 'Content-Type': 'application/xml; charset=utf-8' },
  });
};