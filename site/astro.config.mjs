import { defineConfig } from 'astro/config';
import rehypeScrollable from './scripts/rehype-scrollable.mjs';

export default defineConfig({
  site: 'https://orbiscreen.com',
  trailingSlash: 'never',
  build: {
    format: 'directory',

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

    rehypePlugins: [rehypeScrollable],
  },
  vite: {
    build: {

      chunkSizeWarningLimit: 700,
    },
  },
  devToolbar: {
    enabled: false,
  },
});
