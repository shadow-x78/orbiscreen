// ─────────────────────────────────────────────
// Orbiscreen - Route manifest (drives sitemap, hreflang, checks)
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import docsMeta from './docs-meta.json';

export interface Route {

  path: string;

  arPath: string;

  priority: number;

  changefreq: 'daily' | 'weekly' | 'monthly' | 'yearly';
}

const docs = docsMeta.docs;

const staticRoutes: Route[] = [
  { path: '/', arPath: '/ar', priority: 1.0, changefreq: 'weekly' },
  { path: '/features', arPath: '/ar/features', priority: 0.9, changefreq: 'monthly' },
  { path: '/install', arPath: '/ar/install', priority: 0.9, changefreq: 'weekly' },
  { path: '/compare', arPath: '/ar/compare', priority: 0.8, changefreq: 'monthly' },
  { path: '/faq', arPath: '/ar/faq', priority: 0.8, changefreq: 'monthly' },
  { path: '/docs', arPath: '/ar/docs', priority: 0.9, changefreq: 'weekly' },
  ...docs.map<Route>((doc) => ({
    path: `/docs/${doc.slug}`,
    arPath: `/ar/docs/${doc.slug}`,
    priority: 0.7,
    changefreq: 'monthly',
  })),
  { path: '/changelog', arPath: '/ar/changelog', priority: 0.6, changefreq: 'weekly' },
  { path: '/license', arPath: '/ar/license', priority: 0.4, changefreq: 'yearly' },
  { path: '/security', arPath: '/ar/security', priority: 0.5, changefreq: 'monthly' },
  { path: '/privacy', arPath: '/ar/privacy', priority: 0.3, changefreq: 'yearly' },
];

export const routes = staticRoutes;
