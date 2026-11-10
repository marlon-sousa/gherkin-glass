// Compiled from script/ by cargo xtask build. Do not edit: change script/src and build again.

// src/adapters/dom_page.ts
var DomBlock = class {
  anchor;
  parent;
  button;
  content;
  constructor(heading, button, content, parent) {
    this.anchor = heading.id;
    this.button = button;
    this.content = content;
    this.parent = parent;
  }
  isExpanded() {
    return this.button.getAttribute("aria-expanded") === "true";
  }
  setExpanded(expanded) {
    this.button.setAttribute("aria-expanded", String(expanded));
    this.content.hidden = !expanded;
  }
};
var DomPage = class {
  found = [];
  document;
  constructor(document2) {
    this.document = document2;
    const byElement = /* @__PURE__ */ new Map();
    for (const element of document2.querySelectorAll(".ghg-block")) {
      const heading = element.querySelector(":scope > :is(h2, h3, h4)");
      const button = heading?.querySelector(":scope > button.ghg-toggle");
      const content = element.querySelector(":scope > .ghg-content");
      if (!heading || !button || !content) {
        continue;
      }
      const around = element.parentElement?.closest(".ghg-block");
      const parent = around ? byElement.get(around) ?? null : null;
      const block = new DomBlock(heading, button, content, parent);
      byElement.set(element, block);
      this.found.push(block);
    }
  }
  blocks() {
    return this.found;
  }
  target() {
    return anchorOf(this.document.location.hash);
  }
  onToggle(handler) {
    for (const block of this.found) {
      block.button.addEventListener("click", () => handler(block));
    }
  }
  onCollapseAll(handler) {
    this.onClick(".ghg-collapse-all", handler);
  }
  onExpandAll(handler) {
    this.onClick(".ghg-expand-all", handler);
  }
  onFollow(handler) {
    const view = this.document.defaultView;
    view?.addEventListener("hashchange", () => {
      const anchor = anchorOf(this.document.location.hash);
      if (anchor !== null) {
        handler(anchor);
      }
    });
    this.document.addEventListener("click", (event) => {
      if (event.defaultPrevented || event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) {
        return;
      }
      const target = event.target;
      const link = typeof target?.closest === "function" ? target.closest("a[href^='#']") : null;
      const anchor = link ? anchorOf(link.getAttribute("href") ?? "") : null;
      if (anchor !== null) {
        handler(anchor);
      }
    });
  }
  onClick(selector, handler) {
    this.document.querySelector(selector)?.addEventListener("click", handler);
  }
};
function anchorOf(hash) {
  if (!hash.startsWith("#") || hash.length === 1) {
    return null;
  }
  try {
    return decodeURIComponent(hash.slice(1));
  } catch {
    return hash.slice(1);
  }
}

// src/adapters/local_memory.ts
var LocalMemory = class {
  get(key) {
    try {
      return window.localStorage.getItem(key);
    } catch {
      return null;
    }
  }
  set(key, value) {
    try {
      window.localStorage.setItem(key, value);
    } catch {
    }
  }
};

// src/policies/block_state.ts
var EXPANDED = "expanded";
var COLLAPSED = "collapsed";
function memoryKey(pagePath, anchor) {
  return `gherkin-glass:${pagePath}#${anchor}`;
}
function remembered(expanded) {
  return expanded ? EXPANDED : COLLAPSED;
}
function startState(configured, kept) {
  if (kept === EXPANDED) {
    return true;
  }
  if (kept === COLLAPSED) {
    return false;
  }
  return configured;
}

// src/services/collapsing.ts
function startCollapsing(page, memory, pagePath) {
  const blocks = page.blocks();
  const set = (block, expanded) => {
    block.setExpanded(expanded);
    memory.set(memoryKey(pagePath, block.anchor), remembered(expanded));
  };
  const reveal = (anchor) => {
    let block = blocks.find((found) => found.anchor === anchor) ?? null;
    while (block !== null) {
      if (!block.isExpanded()) {
        set(block, true);
      }
      block = block.parent;
    }
  };
  for (const block of blocks) {
    const kept = memory.get(memoryKey(pagePath, block.anchor));
    block.setExpanded(startState(block.isExpanded(), kept));
  }
  const target = page.target();
  if (target !== null) {
    reveal(target);
  }
  page.onToggle((block) => set(block, !block.isExpanded()));
  page.onCollapseAll(() => blocks.forEach((block) => set(block, false)));
  page.onExpandAll(() => blocks.forEach((block) => set(block, true)));
  page.onFollow(reveal);
}

// src/main.ts
startCollapsing(new DomPage(document), new LocalMemory(), document.location.pathname);
