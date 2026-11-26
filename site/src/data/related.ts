import docsMeta from './docs-meta.json';
import type { Lang } from '../lib/site';

export interface RelatedItem {
  href: string;
  title: string;
  description: string;
}

export type RelatedPage = 'home' | 'features' | 'install' | 'faq';

const prefix = (lang: Lang) => (lang === 'ar' ? '/ar' : '');
const bySlug = new Map(docsMeta.docs.map((doc) => [doc.slug, doc]));

function doc(lang: Lang, slug: string): RelatedItem {
  const entry = bySlug.get(slug);
  if (!entry) throw new Error(`Unknown documentation slug: ${slug}`);
  return {
    href: `${prefix(lang)}/docs/${entry.slug}`,
    title: lang === 'ar' ? entry.titleAr : entry.title,
    description: lang === 'ar' ? entry.descriptionAr : entry.description,
  };
}

function privacy(lang: Lang): RelatedItem {
  return {
    href: `${prefix(lang)}/privacy`,
    title: lang === 'ar' ? 'الخصوصية' : 'Privacy',
    description:
      lang === 'ar'
        ? 'ما يجمعه الموقع والتطبيق، وما لا يرسله الخادم، وكيف تتحقق من ذلك.'
        : 'What the site and the app collect, what the daemon never sends, and how to verify it.',
  };
}

const pages: Record<RelatedPage, (lang: Lang) => RelatedItem[]> = {
  home: (lang) => [
    doc(lang, 'architecture'),
    doc(lang, 'udp-transport'),
    doc(lang, 'de-support'),
    doc(lang, 'packaging'),
  ],
  features: (lang) => [
    doc(lang, 'architecture'),
    doc(lang, 'frame-transport'),
    doc(lang, 'de-support'),
    doc(lang, 'udp-transport'),
  ],
  install: (lang) => [
    doc(lang, 'packaging'),
    doc(lang, 'troubleshooting'),
    doc(lang, 'de-support'),
    doc(lang, 'dbus-spec'),
  ],
  faq: (lang) => [
    doc(lang, 'troubleshooting'),
    doc(lang, 'de-support'),
    doc(lang, 'architecture'),
    privacy(lang),
  ],
};

export function relatedDocs(page: RelatedPage, lang: Lang): RelatedItem[] {
  return pages[page](lang);
}
