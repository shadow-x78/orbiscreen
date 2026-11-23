// ─────────────────────────────────────────────
// Orbiscreen - Markdown mirrors
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import { landing } from '../data/landing';
import { pages } from '../data/pages';
import { installMethods, downloads } from '../data/install';
import { platforms } from '../data/platforms';
import { comparison } from '../data/compare';
import { faq } from '../data/faq';
import { changelogPage, licensePage, privacyPage, securityPage } from '../data/legal';
import docsMeta from '../data/docs-meta.json';
import { CHANGELOG_PAGE_SIZE, getReleases } from './changelog';
import { GITHUB_URL, SITE_URL, type Lang } from './site';

export interface MirrorDoc {
  slug: string;
  lang: Lang;
  title: string;
  description: string;
  body: string;
}

function stripHtml(value: string): string {
  return value.replace(/<code>([^<]+)<\/code>/g, '`$1`').replace(/<[^>]+>/g, '');
}

function bullets(items: string[]): string {
  return items.map((item) => `- ${item}`).join('\n');
}

function home(lang: Lang): string {
  const data = landing[lang];
  const title = lang === 'ar' ? 'Orbiscreen: تابلتك، شاشتك الثانية' : 'Orbiscreen: Your tablet is your second monitor';
  return [
    `# ${title}`,
    '',
    data.hero.sub,
    '',
    '## ' + data.capabilities.title,
    '',
    data.capabilities.tiles.map((tile) => `### ${tile.title}\n\n${tile.body}`).join('\n\n'),
    '',
    '## ' + data.howItWorks.title,
    '',
    data.howItWorks.steps.map((step) => `${step.num}. **${step.title}**: ${step.body} (${step.mono})`).join('\n'),
    '',
    '## ' + data.platform.title,
    '',
    data.platform.sub,
    '',
    data.platform.rows.map((row) => `- ${row.name}: ${row.status}`).join('\n'),
    '',
    '## ' + data.performance.title,
    '',
    data.performance.stats.map((stat) => `- ${stat.value}: ${stat.label}`).join('\n'),
    '',
    '## ' + data.install.title,
    '',
    lang === 'ar' ? `كل الحزم: ${GITHUB_URL}/releases` : `All packages: ${GITHUB_URL}/releases`,
    '',
    '## ' + data.oss.title,
    '',
    data.oss.body,
    '',
  ].join('\n');
}

function features(lang: Lang): string {
  const page = pages.features[lang];
  const sections = page.sections
    .map((section) =>
      [
        `## ${section.title}`,
        '',
        section.body.join('\n\n'),
        '',
        section.points ? bullets(section.points) : '',
        '',
      ].join('\n'),
    )
    .join('\n');
  return [`# ${page.title}`, '', page.lede, '', sections].join('\n');
}

function install(lang: Lang): string {
  const page = pages.install[lang];
  const methods = installMethods[lang]
    .map((method) => {
      const code = method.lines.map((line) => (line.kind === 'comment' ? line.text : line.text)).join('\n');
      return [`## ${method.label}`, '', method.note, '', '```bash', code, '```', ''].join('\n');
    })
    .join('\n');
  const after = page.after.steps.map((step, index) => `${index + 1}. **${step.title}**: ${step.text}`).join('\n');
  const matrix = platforms[lang]
    .map((row) => `| ${row.name} | ${row.virtual} | ${row.capture} | ${row.input} |`)
    .join('\n');
  return [
    `# ${page.title}`,
    '',
    page.lede,
    '',
    `## ${page.requirements.title}`,
    '',
    bullets(page.requirements.items),
    '',
    methods,
    '## ' + page.after.title,
    '',
    after,
    '',
    `## ${page.platformTitle}`,
    '',
    page.platformLede,
    '',
    '| Environment | Virtual display | Capture | Input |',
    '| --- | --- | --- | --- |',
    matrix,
    '',
    '## Downloads',
    '',
    downloads[lang].host.map((item) => `- ${item.main}: ${item.href}`).join('\n'),
    '',
    downloads[lang].clients.map((item) => `- ${item.main}: ${item.href}`).join('\n'),
    '',
  ].join('\n');
}

function compare(lang: Lang): string {
  const page = pages.compare[lang];
  const header = '| Capability | Spacedesk | Deskreen | Weylus | Sidecar | Orbiscreen |\n| --- | --- | --- | --- | --- | --- |';
  const rows = comparison[lang]
    .map((row) => `| ${row.capability} | ${row.spacedesk} | ${row.deskreen} | ${row.weylus} | ${row.sidecar} | ${row.orbiscreen} |`)
    .join('\n');
  return [
    `# ${page.title}`,
    '',
    page.lede,
    '',
    header,
    rows,
    '',
    `## ${page.methodology.title}`,
    '',
    page.methodology.body.join('\n\n'),
    '',
    page.differences.map((item) => `## ${item.title}\n\n${item.body}`).join('\n\n'),
    '',
  ].join('\n');
}

function faqs(lang: Lang): string {
  const page = pages.faq[lang];
  const items = faq[lang]
    .map((item) => `## ${item.question}\n\n${item.answer.map(stripHtml).join(' ')}`)
    .join('\n\n');
  return [`# ${page.title}`, '', page.lede, '', items, ''].join('\n');
}

function changelog(lang: Lang): string {
  const page = changelogPage[lang];
  const releases = getReleases()
    .slice(0, CHANGELOG_PAGE_SIZE)
    .map((release) => {
      const body = release.blocks
        .map((block) => {
          if (block.type === 'heading') return `### ${block.text}`;
          if (block.type === 'list') {
            return block.items
              .map((item) => `${'  '.repeat(item.depth)}- ${item.text}`)
              .join('\n');
          }
          return block.text;
        })
        .join('\n\n');
      return `## ${release.tag}${release.date ? ` - ${release.date}` : ''}\n\n${body}`;
    })
    .join('\n\n');
  return [`# ${page.title}`, '', page.lede, '', releases, ''].join('\n');
}

function simple(page: {
  title: string;
  lede: string;
  sections: { title: string; body: string[]; list?: string[] }[];
}): string {
  return [
    `# ${page.title}`,
    '',
    page.lede,
    '',
    page.sections
      .map((section) => `## ${section.title}\n\n${section.body.join('\n\n')}${section.list ? `\n\n${bullets(section.list)}` : ''}`)
      .join('\n\n'),
    '',
  ].join('\n');
}

function docsIndex(lang: Lang): string {
  const list = docsMeta.docs
    .map((doc) => {
      const title = lang === 'ar' ? doc.titleAr : doc.title;
      const description = lang === 'ar' ? doc.descriptionAr : doc.description;
      return `- [${title}](${SITE_URL}${lang === 'ar' ? '/ar' : ''}/docs/${doc.slug}): ${description}`;
    })
    .join('\n');
  return [`# ${lang === 'ar' ? 'التوثيق' : 'Documentation'}`, '', list, ''].join('\n');
}

export function buildMirror(requested: string, docs: MirrorDoc[]): string | null {
  const path = requested.replace(/\.md$/, '').replace(/^\/+|\/+$/g, '');
  const isAr = path === 'ar' || path.startsWith('ar/');
  const lang: Lang = isAr ? 'ar' : 'en';
  const key = isAr ? path.replace(/^ar\/?/, '') : path;

  if (key === '' || key === 'index') return home(lang);
  if (key === 'features') return features(lang);
  if (key === 'install') return install(lang);
  if (key === 'compare') return compare(lang);
  if (key === 'faq') return faqs(lang);
  if (key === 'changelog') return changelog(lang);
  if (key === 'license') return simple(licensePage[lang]);
  if (key === 'security') return simple(securityPage[lang]);
  if (key === 'privacy') return simple(privacyPage[lang]);
  if (key === 'docs') return docsIndex(lang);

  const docMatch = key.match(/^docs\/([a-z0-9-]+)$/);
  if (docMatch) {
    const doc = docs.find((entry) => entry.lang === lang && entry.slug === docMatch[1]);
    if (!doc) return null;
    return `# ${doc.title}\n\n${doc.description}\n\n${doc.body.trim()}\n`;
  }

  return null;
}

export function allMirrorPaths(docs: MirrorDoc[]): string[] {
  const bases = ['index', 'features', 'install', 'compare', 'faq', 'changelog', 'license', 'security', 'privacy', 'docs'];
  const paths: string[] = [];
  for (const base of bases) {
    paths.push(`${base}.md`);
    paths.push(`ar/${base}.md`);
  }
  for (const doc of docs) {
    paths.push(doc.lang === 'ar' ? `ar/docs/${doc.slug}.md` : `docs/${doc.slug}.md`);
  }
  return [...new Set(paths)];
}