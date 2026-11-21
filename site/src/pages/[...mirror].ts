import type { APIRoute, GetStaticPaths } from 'astro';
import { getCollection } from 'astro:content';
import { allMirrorPaths, buildMirror, type MirrorDoc } from '../lib/mirror';

export const getStaticPaths = (async () => {
  const entries = await getCollection('docs');
  const docs: MirrorDoc[] = entries.map((entry) => ({
    slug: entry.data.slug,
    lang: entry.data.lang,
    title: entry.data.title,
    description: entry.data.description,
    body: entry.body ?? '',
  }));

  return allMirrorPaths(docs).map((path) => ({
    params: { mirror: path },
    props: { docs },
  }));
}) satisfies GetStaticPaths;

export const GET: APIRoute = ({ params, props }) => {
  const docs = (props as { docs: MirrorDoc[] }).docs;
  const requested = params.mirror ?? '';
  const markdown = buildMirror(requested, docs);

  if (!markdown) {
    return new Response('Not found\n', {
      status: 404,
      headers: { 'Content-Type': 'text/plain; charset=utf-8' },
    });
  }

  return new Response(markdown, {
    headers: { 'Content-Type': 'text/markdown; charset=utf-8' },
  });
};