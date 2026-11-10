// Compiled from script/ by cargo xtask build. Do not edit: change script/src and build again.

// src/adapters/dom_backgrounds.ts
var DomBackgrounds = class _DomBackgrounds {
  checkbox;
  document;
  constructor(document2, checkbox) {
    this.document = document2;
    this.checkbox = checkbox;
  }
  /** Null on a page without the checkbox, where no scenario starts with a background's steps. */
  static of(document2) {
    const checkbox = document2.querySelector("input.ghg-inline-backgrounds");
    return checkbox ? new _DomBackgrounds(document2, checkbox) : null;
  }
  writtenInline() {
    return this.checkbox.defaultChecked;
  }
  setInline(inline) {
    this.checkbox.checked = inline;
    for (const steps of this.document.querySelectorAll(".ghg-from-background > .ghg-inline")) {
      steps.hidden = !inline;
    }
    for (const link of this.document.querySelectorAll(".ghg-from-background .ghg-linked")) {
      link.hidden = inline;
    }
  }
  onSwitch(handler) {
    this.checkbox.addEventListener("change", () => handler(this.checkbox.checked));
  }
  onReturn(handler) {
    this.document.defaultView?.addEventListener("pageshow", (event) => {
      if (event.persisted) {
        handler();
      }
    });
  }
};

// src/adapters/dom_page.ts
var DomBlock = class {
  anchor;
  parent;
  button;
  content;
  constructor(anchor, button, content, parent) {
    this.anchor = anchor;
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
      const block = new DomBlock(heading.id, button, content, parent);
      byElement.set(element, block);
      this.found.push(block);
    }
    for (const element of document2.querySelectorAll(".ghg-feature")) {
      const link = element.querySelector(":scope > a[href]");
      const button = element.querySelector(":scope > button.ghg-toggle");
      const content = element.querySelector(":scope > .ghg-content");
      if (!link || !button || !content) {
        continue;
      }
      this.found.push(new DomBlock(link.getAttribute("href") ?? "", button, content, null));
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
      const link = typeof target?.closest === "function" ? target.closest("a[href]") : null;
      const anchor = link ? this.anchorOnPage(link) : null;
      if (anchor !== null) {
        handler(anchor);
      }
    });
  }
  /** The anchor a link leads to on this very page, such as the folder tree's, whatever its address is relative to. */
  anchorOnPage(link) {
    let address;
    try {
      address = new URL(link.getAttribute("href") ?? "", this.document.location.href);
    } catch {
      return null;
    }
    const here = this.document.location;
    return address.origin === here.origin && address.pathname === here.pathname ? anchorOf(address.hash) : null;
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

// src/policies/tree_state.ts
var NARROW = "not all and (min-width: 48em)";
var SHOWN = "shown";
var HIDDEN = "hidden";
var EXPANDED = "expanded";
var COLLAPSED = "collapsed";
function shownKey(sitePath2) {
  return `gherkin-glass:${sitePath2} tree`;
}
function itemKey(sitePath2, address) {
  return `gherkin-glass:${sitePath2} tree ${address}`;
}
function rememberedShown(shown) {
  return shown ? SHOWN : HIDDEN;
}
function rememberedExpanded(expanded) {
  return expanded ? EXPANDED : COLLAPSED;
}
function startShown(configured, kept, narrow) {
  if (kept === SHOWN) {
    return true;
  }
  if (kept === HIDDEN) {
    return false;
  }
  return configured && !narrow;
}
function startExpanded(onPath, kept) {
  return onPath || kept === EXPANDED;
}
function pageAddress(sitePath2, pagePath) {
  const site2 = decoded(sitePath2);
  const page2 = decoded(pagePath);
  if (!page2.startsWith(site2)) {
    return null;
  }
  const address = page2.slice(site2.length);
  return address === "" || address.endsWith("/") ? `${address}index.html` : address;
}
function rootFrom(address) {
  return "../".repeat(address.split("/").length - 1);
}
function pathTo(items, address) {
  for (const item of items) {
    if (!item.address.includes("#") && decoded(item.address) === address) {
      return [item.address];
    }
    const inner = pathTo(item.items ?? [], address);
    if (inner.length > 0) {
      return [...inner, item.address];
    }
  }
  return [];
}
function readTree(data) {
  return isItems(data) ? data : null;
}
function isItems(data) {
  return Array.isArray(data) && data.every(isItem);
}
function isItem(data) {
  if (typeof data !== "object" || data === null) {
    return false;
  }
  const item = data;
  return typeof item["text"] === "string" && typeof item["address"] === "string" && (item["lang"] === void 0 || typeof item["lang"] === "string") && (item["items"] === void 0 || isItems(item["items"]));
}
function decoded(path) {
  try {
    return decodeURIComponent(path);
  } catch {
    return path;
  }
}

// src/adapters/dom_tree.ts
var DomTreeSwitch = class {
  checkbox;
  landmark;
  constructor(document2) {
    this.checkbox = document2.querySelector("input.ghg-show-tree");
    this.landmark = document2.querySelector("nav.ghg-tree");
  }
  configured() {
    return this.checkbox?.checked ?? false;
  }
  setShown(shown) {
    if (this.checkbox) {
      this.checkbox.checked = shown;
    }
    if (this.landmark) {
      this.landmark.hidden = !shown;
    }
  }
  disable() {
    this.setShown(false);
    if (this.checkbox) {
      this.checkbox.disabled = true;
    }
  }
  onChange(handler) {
    const checkbox = this.checkbox;
    checkbox?.addEventListener("change", () => handler(checkbox.checked));
  }
};
var DomTree = class {
  document;
  landmark;
  /** What an item's address is relative to: the way back to the site's root, or the root itself. */
  root;
  nodes = [];
  byLink = /* @__PURE__ */ new Map();
  handler = () => {
  };
  /** `page` is the current page's address below the site's root, `site`, decoded, if it is below it. */
  constructor(document2, site2, page2) {
    this.document = document2;
    this.landmark = document2.querySelector("nav.ghg-tree");
    this.root = page2 === null ? site2 : rootFrom(page2);
  }
  build(items, current, expanded) {
    if (!this.landmark) {
      return;
    }
    const tree = this.document.createElement("ul");
    tree.setAttribute("role", "tree");
    tree.className = "ghg-treeview";
    tree.setAttribute("aria-label", this.landmark.getAttribute("aria-label") ?? "");
    this.add(tree, items, null, current, expanded);
    this.landmark.replaceChildren(tree);
    const stop = this.nodes.find((node) => node.address === current) ?? this.nodes[0];
    if (stop) {
      stop.link.tabIndex = 0;
    }
    tree.addEventListener("keydown", (event) => this.onKey(event));
  }
  onToggle(handler) {
    this.handler = handler;
  }
  add(list, items, parent, current, expanded) {
    for (const item of items) {
      const entry = this.document.createElement("li");
      entry.setAttribute("role", "none");
      if (item.lang !== void 0) {
        entry.lang = item.lang;
      }
      const link = this.document.createElement("a");
      link.setAttribute("role", "treeitem");
      link.href = typeof this.root === "string" ? this.root + item.address : new URL(item.address, this.root).href;
      link.tabIndex = -1;
      if (item.address === current) {
        link.setAttribute("aria-current", "page");
      }
      const inner = item.items ?? [];
      let group = null;
      if (inner.length > 0) {
        const marker = this.document.createElement("span");
        marker.className = "ghg-marker";
        marker.setAttribute("aria-hidden", "true");
        link.append(marker);
        group = this.document.createElement("ul");
        group.setAttribute("role", "group");
        group.id = `ghg_tree_${this.nodes.length}`;
        group.setAttribute("aria-label", item.text);
        link.setAttribute("aria-owns", group.id);
      }
      link.append(item.text);
      entry.append(link);
      const node = { link, address: item.address, group, parent };
      this.nodes.push(node);
      this.byLink.set(link, node);
      if (group) {
        entry.append(group);
        this.setExpanded(node, expanded(item.address));
        link.querySelector(".ghg-marker")?.addEventListener("click", (event) => {
          event.preventDefault();
          event.stopPropagation();
          this.focus(node);
          this.toggle(node, !this.isExpanded(node));
        });
        this.add(group, inner, node, current, expanded);
      }
      list.append(entry);
    }
  }
  isExpanded(node) {
    return node.link.getAttribute("aria-expanded") === "true";
  }
  setExpanded(node, expanded) {
    if (node.group) {
      node.link.setAttribute("aria-expanded", String(expanded));
      node.group.hidden = !expanded;
    }
  }
  /** As the reader asked, and remembered. */
  toggle(node, expanded) {
    if (node.group && this.isExpanded(node) !== expanded) {
      this.setExpanded(node, expanded);
      this.handler(node.address, expanded);
    }
  }
  /** The items not inside a collapsed item, in the order of the tree. */
  visible() {
    return this.nodes.filter((node) => {
      for (let above = node.parent; above !== null; above = above.parent) {
        if (!this.isExpanded(above)) {
          return false;
        }
      }
      return true;
    });
  }
  focus(node) {
    for (const other of this.nodes) {
      other.link.tabIndex = other === node ? 0 : -1;
    }
    node.link.focus();
  }
  onKey(event) {
    if (event.altKey || event.ctrlKey || event.metaKey) {
      return;
    }
    const target = event.target;
    const node = target ? this.byLink.get(target) : void 0;
    if (!node) {
      return;
    }
    const visible = this.visible();
    const at = visible.indexOf(node);
    const move = (to) => {
      if (to) {
        this.focus(to);
      }
    };
    switch (event.key) {
      case "ArrowDown":
        move(visible[at + 1]);
        break;
      case "ArrowUp":
        move(visible[at - 1]);
        break;
      case "ArrowRight":
        if (node.group && !this.isExpanded(node)) {
          this.toggle(node, true);
        } else if (node.group) {
          move(visible[at + 1]);
        }
        break;
      case "ArrowLeft":
        if (node.group && this.isExpanded(node)) {
          this.toggle(node, false);
        } else if (node.parent) {
          move(node.parent);
        }
        break;
      case "Home":
        move(visible[0]);
        break;
      case "End":
        move(visible[visible.length - 1]);
        break;
      case " ":
        node.link.click();
        break;
      case "*":
        for (const sibling of this.nodes) {
          if (sibling.parent === node.parent) {
            this.toggle(sibling, true);
          }
        }
        break;
      default:
        if ([...event.key].length === 1 && /^[\p{L}\p{N}]$/u.test(event.key)) {
          const wanted = event.key.toLocaleLowerCase();
          const after = [...visible.slice(at + 1), ...visible.slice(0, at + 1)];
          move(after.find((other) => (other.link.textContent ?? "").trim().toLocaleLowerCase().startsWith(wanted)));
          break;
        }
        return;
    }
    event.preventDefault();
    event.stopPropagation();
  }
};

// src/adapters/fetched_tree.ts
var FetchedTree = class {
  address;
  fetcher;
  /** Chromium, through Playwright, refuses `fetch` called as a method of another object, so it is called on its own. */
  constructor(address, fetcher = (input, init) => fetch(input, init)) {
    this.address = address;
    this.fetcher = fetcher;
  }
  async load() {
    try {
      const response = await this.fetcher(this.address);
      return response.ok ? readTree(await response.json()) : null;
    } catch {
      return null;
    }
  }
};

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

// src/policies/background_form.ts
var INLINE = "inline";
var LINKED = "linked";
function siteRoot(scriptAddress) {
  return new URL("..", scriptAddress).pathname;
}
function formKey(siteRoot2) {
  return `gherkin-glass:${siteRoot2}:backgrounds`;
}
function rememberedForm(inline) {
  return inline ? INLINE : LINKED;
}
function startForm(configured, kept) {
  if (kept === INLINE) {
    return true;
  }
  if (kept === LINKED) {
    return false;
  }
  return configured;
}

// src/services/backgrounds.ts
function startBackgrounds(backgrounds2, memory2, siteRoot2) {
  const key = formKey(siteRoot2);
  const show = () => backgrounds2.setInline(startForm(backgrounds2.writtenInline(), memory2.get(key)));
  show();
  backgrounds2.onSwitch((inline) => {
    backgrounds2.setInline(inline);
    memory2.set(key, rememberedForm(inline));
  });
  backgrounds2.onReturn(show);
}

// src/policies/block_state.ts
var EXPANDED2 = "expanded";
var COLLAPSED2 = "collapsed";
function memoryKey(pagePath, anchor) {
  return `gherkin-glass:${pagePath}#${anchor}`;
}
function remembered(expanded) {
  return expanded ? EXPANDED2 : COLLAPSED2;
}
function startState(configured, kept) {
  if (kept === EXPANDED2) {
    return true;
  }
  if (kept === COLLAPSED2) {
    return false;
  }
  return configured;
}

// src/services/collapsing.ts
function startCollapsing(page2, memory2, pagePath) {
  const blocks = page2.blocks();
  const set = (block, expanded) => {
    block.setExpanded(expanded);
    memory2.set(memoryKey(pagePath, block.anchor), remembered(expanded));
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
    const kept = memory2.get(memoryKey(pagePath, block.anchor));
    block.setExpanded(startState(block.isExpanded(), kept));
  }
  const target = page2.target();
  if (target !== null) {
    reveal(target);
  }
  page2.onToggle((block) => set(block, !block.isExpanded()));
  page2.onCollapseAll(() => blocks.forEach((block) => set(block, false)));
  page2.onExpandAll(() => blocks.forEach((block) => set(block, true)));
  page2.onFollow(reveal);
}

// src/services/folder_tree.ts
async function startFolderTree(source, toggle, view, memory2, place) {
  const shown = startShown(toggle.configured(), memory2.get(shownKey(place.sitePath)), place.narrow);
  toggle.setShown(shown);
  toggle.onChange((chosen) => {
    toggle.setShown(chosen);
    memory2.set(shownKey(place.sitePath), rememberedShown(chosen));
  });
  const items = await source.load();
  if (items === null) {
    toggle.disable();
    return;
  }
  const path = place.page === null ? [] : pathTo(items, place.page);
  view.build(items, path[0] ?? null, (address) => startExpanded(path.includes(address), memory2.get(itemKey(place.sitePath, address))));
  view.onToggle((address, expanded) => memory2.set(itemKey(place.sitePath, address), rememberedExpanded(expanded)));
}

// src/main.ts
var memory = new LocalMemory();
startCollapsing(new DomPage(document), memory, document.location.pathname);
var site = new URL("../", import.meta.url);
var sitePath = site.pathname;
var page = pageAddress(sitePath, document.location.pathname);
void startFolderTree(new FetchedTree(new URL("tree.json", import.meta.url)), new DomTreeSwitch(document), new DomTree(document, site, page), memory, {
  sitePath,
  page,
  narrow: window.matchMedia(NARROW).matches
});
var backgrounds = DomBackgrounds.of(document);
if (backgrounds !== null) {
  startBackgrounds(backgrounds, memory, siteRoot(import.meta.url));
}
