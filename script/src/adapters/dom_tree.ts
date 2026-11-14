// Adapter: the tree view over the page's markup, as src/policies/rendering/page_layout.html writes it, the tree built as the WAI-ARIA Navigation Treeview example builds it.

import { rootFrom } from "../policies/tree_state.ts";
import type { TreeItem, TreeView } from "../ports/tree.ts";

/** An item of the tree as the page shows it. */
interface TreeNode {
  readonly link: HTMLAnchorElement;
  readonly address: string;
  /** The list of the items inside it, if any. */
  readonly group: HTMLUListElement | null;
  readonly parent: TreeNode | null;
}

export class DomTree implements TreeView {
  private readonly document: Document;
  private readonly landmark: HTMLElement | null;
  /** What an item's address is relative to: the way back to the site's root, or the root itself. */
  private readonly root: string | URL;
  private readonly nodes: TreeNode[] = [];
  private readonly byLink = new Map<Element, TreeNode>();
  private handler: (address: string, expanded: boolean) => void = () => {};

  /** `page` is the current page's address below the site's root, `site`, decoded, if it is below it. */
  constructor(document: Document, site: URL, page: string | null) {
    this.document = document;
    this.landmark = document.querySelector<HTMLElement>("nav.ghg-tree");
    // Relative when the page is in the site, so that the links are those the page would have.
    this.root = page === null ? site : rootFrom(page);
  }

  build(items: readonly TreeItem[], current: string | null, expanded: (address: string) => boolean): void {
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

  onToggle(handler: (address: string, expanded: boolean) => void): void {
    this.handler = handler;
  }

  /** `lang` is the feature's that holds `items`, if one does. */
  private add(list: HTMLUListElement, items: readonly TreeItem[], parent: TreeNode | null, current: string | null, expanded: (address: string) => boolean, lang?: string): void {
    for (const item of items) {
      const itemLang = item.lang ?? lang;
      const entry = this.document.createElement("li");
      entry.setAttribute("role", "none");
      const link = this.document.createElement("a");
      link.setAttribute("role", "treeitem");
      link.href = typeof this.root === "string" ? this.root + item.address : new URL(item.address, this.root).href;
      link.tabIndex = -1;
      if (item.address === current) {
        link.setAttribute("aria-current", "page");
      }
      const inner = item.items ?? [];
      let group: HTMLUListElement | null = null;
      if (inner.length > 0) {
        const marker = this.document.createElement("span");
        marker.className = "ghg-marker";
        marker.setAttribute("aria-hidden", "true");
        link.append(marker);
        group = this.document.createElement("ul");
        group.setAttribute("role", "group");
        group.id = `ghg_tree_${this.nodes.length}`;
        group.setAttribute("aria-label", textOf(item));
        link.setAttribute("aria-owns", group.id);
      }
      link.append(...this.words(item, itemLang));
      entry.append(link);
      const node: TreeNode = { link, address: item.address, group, parent };
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
        this.add(group, inner, node, current, expanded, itemLang);
      }
      list.append(entry);
    }
  }

  /**
   * The item's words, each run of feature text isolated in a `bdi` with its
   * feature's `lang`, so that the tree's own words are read in the page's.
   */
  private words(item: TreeItem, lang: string | undefined): (string | HTMLElement)[] {
    if (typeof item.text === "string") {
      return [item.text];
    }
    return item.text.map((run, place) => {
      if (place % 2 === 0) {
        return run;
      }
      const isolated = this.document.createElement("bdi");
      isolated.append(run);
      if (lang !== undefined) {
        isolated.lang = lang;
      }
      return isolated;
    });
  }

  private isExpanded(node: TreeNode): boolean {
    return node.link.getAttribute("aria-expanded") === "true";
  }

  private setExpanded(node: TreeNode, expanded: boolean): void {
    if (node.group) {
      node.link.setAttribute("aria-expanded", String(expanded));
      node.group.hidden = !expanded;
    }
  }

  /** As the reader asked, and remembered. */
  private toggle(node: TreeNode, expanded: boolean): void {
    if (node.group && this.isExpanded(node) !== expanded) {
      this.setExpanded(node, expanded);
      this.handler(node.address, expanded);
    }
  }

  /** The items not inside a collapsed item, in the order of the tree. */
  private visible(): TreeNode[] {
    return this.nodes.filter((node) => {
      for (let above = node.parent; above !== null; above = above.parent) {
        if (!this.isExpanded(above)) {
          return false;
        }
      }
      return true;
    });
  }

  private focus(node: TreeNode): void {
    for (const other of this.nodes) {
      other.link.tabIndex = other === node ? 0 : -1;
    }
    node.link.focus();
  }

  private onKey(event: KeyboardEvent): void {
    if (event.altKey || event.ctrlKey || event.metaKey) {
      return;
    }
    const target = event.target as Element | null;
    const node = target ? this.byLink.get(target) : undefined;
    if (!node) {
      return;
    }
    const visible = this.visible();
    const at = visible.indexOf(node);
    const move = (to: TreeNode | undefined): void => {
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
        // Enter, and every other key, does what it does on a link.
        return;
    }
    event.preventDefault();
    event.stopPropagation();
  }
}

/** The item's words as plain text, for a name that holds no markup. */
function textOf(item: TreeItem): string {
  return typeof item.text === "string" ? item.text : item.text.join("");
}
