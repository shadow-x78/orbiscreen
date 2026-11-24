// ─────────────────────────────────────────────
// Orbiscreen - Content collections
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { glob } from 'astro/loaders';

const docs = defineCollection({
  loader: glob({
    pattern: '**/*.md',
    base: './src/content/docs',
    generateId: ({ entry }) => entry.replace(/\.md$/, ''),
  }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    lang: z.enum(['en', 'ar']),
    slug: z.string(),
    source: z.string(),
    updated: z.string(),
  }),
});

export const collections = { docs };