import type { APIRoute } from 'astro';
import { CHANGELOG_PAGE_SIZE, getReleases } from '../lib/changelog';
import { SITE_URL, SITE_NAME, SHORT_DESCRIPTION } from '../lib/site';

function escapeXml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

export const GET: APIRoute = () => {
  const all = getReleases();
  const releases = all.slice(0, CHANGELOG_PAGE_SIZE);
  // A release without a valid date would get the build time as pubDate, which
  // makes every deploy look like a new release. Fall back to the next older
  // release that does have a date.
  const fallback = all.find((release) => /^\d{4}-\d{2}-\d{2}$/.test(release.date))?.date;

  const items = releases
    .map((release) => {
      const link = `${SITE_URL}/changelog#${release.anchor}`;
      const rawDate = /^\d{4}-\d{2}-\d{2}$/.test(release.date) ? release.date : fallback;
      const date = rawDate ? new Date(`${rawDate}T00:00:00Z`) : null;
      if (!date || Number.isNaN(date.getTime())) return null;
      const summary = release.text.slice(0, 600);
      return [
        '    <item>',
        `      <title>${escapeXml(`${SITE_NAME} ${release.tag}`)}</title>`,
        `      <link>${escapeXml(link)}</link>`,
        `      <guid isPermaLink="true">${escapeXml(link)}</guid>`,
        `      <pubDate>${date.toUTCString()}</pubDate>`,
        `      <description>${escapeXml(summary)}</description>`,
        '    </item>',
      ].join('\n');
    })
    .filter(Boolean)
    .join('\n');

  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>${escapeXml(`${SITE_NAME} releases`)}</title>
    <link>${SITE_URL}/changelog</link>
    <atom:link href="${SITE_URL}/rss.xml" rel="self" type="application/rss+xml"/>
    <description>${escapeXml(SHORT_DESCRIPTION)}</description>
    <language>en</language>
    <lastBuildDate>${new Date().toUTCString()}</lastBuildDate>
${items}
  </channel>
</rss>
`;

  return new Response(xml, {
    headers: { 'Content-Type': 'application/rss+xml; charset=utf-8' },
  });
};