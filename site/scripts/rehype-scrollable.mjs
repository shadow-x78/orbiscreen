// ─────────────────────────────────────────────
// Orbiscreen - make scrollable prose widgets reachable by keyboard
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────


const CONTAINERS = new Set(['pre', 'table']);

function visit(node) {
  if (!node || typeof node !== 'object') return;

  if (node.type === 'element' && CONTAINERS.has(node.tagName)) {
    node.properties = node.properties ?? {};

    node.properties.tabIndex = 0;
  }

  for (const child of node.children ?? []) visit(child);
}

export default function rehypeScrollable() {
  return (tree) => {
    visit(tree);
  };
}