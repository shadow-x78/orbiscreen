// ─────────────────────────────────────────────
// Orbiscreen - make scrollable prose widgets reachable by keyboard
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────
// A horizontally scrollable <pre> or <table> inside a documentation page can be
// read with a mouse but not with a keyboard, because the scroll container is not
// focusable (WCAG 2.1.1). The rendered Markdown comes from the repository's own
// docs, so the attribute is added while the tree is built rather than by
// touching every source file.

const CONTAINERS = new Set(['pre', 'table']);

function visit(node) {
  if (!node || typeof node !== 'object') return;

  if (node.type === 'element' && CONTAINERS.has(node.tagName)) {
    node.properties = node.properties ?? {};
    node.properties.tabIndex = 0;
    // A focusable region needs a name, or a screen reader announces an
    // unlabelled "group" when the user tabs into it.
    if (node.tagName === 'pre') {
      node.properties.role = 'region';
      node.properties.ariaLabel = 'Code sample, scrollable';
    } else {
      node.properties.role = 'region';
      node.properties.ariaLabel = 'Table, scrollable';
    }
  }

  for (const child of node.children ?? []) visit(child);
}

export default function rehypeScrollable() {
  return (tree) => {
    visit(tree);
  };
}