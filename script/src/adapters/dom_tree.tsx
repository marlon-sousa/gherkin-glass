// Adapter: the tree view over the shell's markup, as src/policies/rendering/tree_landmark.html writes it, the tree built as the WAI-ARIA Navigation Treeview example builds it.

import { drawn } from "./jsx/drawing.ts";
import type { Child, Markup } from "./jsx/jsx_runtime.ts";
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
  private readonly landmark: HTMLElement | null;
  private readonly nodes: TreeNode[] = [];
  private readonly byLink = new Map<Element, TreeNode>();
  private handler: (address: string, expanded: boolean) => void = () => {};

  /** The tree of a shell, whose items' addresses are the shell's own, from its `#`. */
  constructor(private readonly document: Document) {
    this.landmark = document.querySelector<HTMLElement>("nav.ghg-tree");
  }

  build(items: readonly TreeItem[], current: string | null, expanded: (address: string) => boolean): void {
    if (!this.landmark) {
      return;
    }
    // A shell builds its tree again for each page it draws.
    this.nodes.length = 0;
    this.byLink.clear();
    this.landmark.replaceChildren(
      ...drawn(
        this.document,
        <ul role="tree" class="ghg-treeview" aria-label={this.landmark.getAttribute("aria-label") ?? ""}>
          {treeEntries(items, { current, expanded, drawnItems: 0 })}
        </ul>,
      ),
    );
    const tree = this.landmark.querySelector<HTMLUListElement>(":scope > ul");
    for (const link of tree?.querySelectorAll<HTMLAnchorElement>("a[role='treeitem']") ?? []) {
      this.add(link);
    }
    const stop = this.nodes.find((node) => node.address === current) ?? this.nodes[0];
    if (stop) {
      stop.link.tabIndex = 0;
    }
    tree?.addEventListener("keydown", (event) => this.onKey(event));
  }

  onToggle(handler: (address: string, expanded: boolean) => void): void {
    this.handler = handler;
  }

  /** Keeps the item `link` draws, the items holding it being kept before it. */
  private add(link: HTMLAnchorElement): void {
    const entry = link.parentElement;
    const holder = entry?.parentElement?.closest("li")?.querySelector(":scope > a");
    const parent = holder ? this.byLink.get(holder) : undefined;
    const node: TreeNode = { link, address: link.getAttribute("href") ?? "", group: entry?.querySelector<HTMLUListElement>(":scope > ul") ?? null, parent: parent ?? null };
    this.nodes.push(node);
    this.byLink.set(link, node);
    link.querySelector(".ghg-marker")?.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      this.focus(node);
      this.toggle(node, !this.isExpanded(node));
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

/** What the items of a tree are drawn from, with the count of those drawn so far, which numbers the next one's group. */
interface Drawing {
  readonly current: string | null;
  readonly expanded: (address: string) => boolean;
  drawnItems: number;
}

/** `lang` is the feature's that holds `items`, if one does. */
function treeEntries(items: readonly TreeItem[], drawing: Drawing, lang?: string): Markup[] {
  return items.map((item) => {
    const itemLang = item.lang ?? lang;
    const inner = item.items ?? [];
    const group = inner.length > 0 ? `ghg_tree_${drawing.drawnItems}` : null;
    const open = group !== null && drawing.expanded(item.address);
    drawing.drawnItems += 1;
    return (
      <li role="none">
        <a role="treeitem" href={item.address} tabindex="-1" aria-current={item.address === drawing.current && "page"} aria-owns={group} aria-expanded={group === null ? null : open ? "true" : "false"}>
          {group !== null && <span class="ghg-marker" aria-hidden="true" />}
          {wordsOf(item, itemLang)}
        </a>
        {group !== null && (
          <ul role="group" id={group} aria-label={textOf(item)} hidden={!open}>
            {treeEntries(inner, drawing, itemLang)}
          </ul>
        )}
      </li>
    );
  });
}

/** The item's words, each run of feature text isolated in a `bdi` with its feature's `lang`, so that the tree's own words are read in the page's. */
function wordsOf(item: TreeItem, lang: string | undefined): Child {
  if (typeof item.text === "string") {
    return item.text;
  }
  return item.text.map((run, place) => (place % 2 === 0 ? run : <bdi lang={lang ?? null}>{run}</bdi>));
}

/** The item's words as plain text, for a name that holds no markup. */
function textOf(item: TreeItem): string {
  return typeof item.text === "string" ? item.text : item.text.join("");
}
