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
    // tabindex alone is what the rule asks for. A region role here would create
    // one landmark per code block, all with the same name, which is its own
    // violation.
    node.properties.tabIndex = 0;
  }

  for (const child of node.children ?? []) visit(child);
}

export default function rehypeScrollable() {
  return (tree) => {
    visit(tree);
  };
}