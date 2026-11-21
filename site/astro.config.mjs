import { defineConfig } from 'astro/config';
import rehypeScrollable from './scripts/rehype-scrollable.mjs';

export default defineConfig({
  site: 'https://orbiscreen.com',
  trailingSlash: 'never',
  build: {
    format: 'directory',
    // Inline small component stylesheets (the install tabs sheet is ~355 B) so
    // they do not cost a render-blocking round trip on the LCP path.
    inlineStylesheets: 'auto',
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'ar'],
    routing: {
      prefixDefaultLocale: false,
      redirectToDefaultLocale: false,
    },
  },
  markdown: {
    syntaxHighlight: false,
    // Scrollable code blocks and tables must be reachable with the keyboard.
    rehypePlugins: [rehypeScrollable],
  },
  vite: {
    build: {
      // The only chunks above Vite's 500 kB default are Mermaid's renderer and
      // its flowchart engine. Both are imported dynamically by DocLayout and
      // fetched only on a documentation page that actually contains a diagram
      // (two pages today), so the warning is noise rather than a regression.
      chunkSizeWarningLimit: 700,
    },
  },
  devToolbar: {
    enabled: false,
  },
});
