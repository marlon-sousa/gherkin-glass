// Adapter: the elements a page is drawn with, made with `createElement` and their text with text nodes, so that no feature text is ever read as markup.

import type { Parts } from "../ports/words.ts";

/** An attribute's value: text, `true` for one with no value, or `null` and `false` for none. */
export type AttributeValue = string | boolean | null;

/** A node, text made into a text node, or `null` and `false` for nothing. */
export type Child = Node | string | null | false;

export class ElementBuilder {
  readonly document: Document;

  constructor(document: Document) {
    this.document = document;
  }

  make<Tag extends keyof HTMLElementTagNameMap>(tag: Tag, attributes: Readonly<Record<string, AttributeValue>> = {}, ...children: readonly Child[]): HTMLElementTagNameMap[Tag] {
    const made = this.document.createElement(tag);
    for (const [name, value] of Object.entries(attributes)) {
      if (value === true) {
        made.setAttribute(name, "");
      } else if (typeof value === "string") {
        made.setAttribute(name, value);
      }
    }
    made.append(...this.nodes(children));
    return made;
  }

  /** `children` as nodes, text as text nodes, with nothing for `null` and `false`. */
  nodes(children: readonly Child[]): Node[] {
    return children.flatMap((child) => {
      if (child === null || child === false) {
        return [];
      }
      return [typeof child === "string" ? this.document.createTextNode(child) : child];
    });
  }

  /** `parts`, each isolated one in a `bdi` of its own, with `lang` when it is given. */
  words(parts: Parts, lang: string | null = null): Node[] {
    return parts.map((part) => (part.isolated ? this.make("bdi", { lang }, part.text) : this.document.createTextNode(part.text)));
  }

  /** Several nodes as one child, such as a heading's words. */
  fragment(...children: readonly Child[]): DocumentFragment {
    const fragment = this.document.createDocumentFragment();
    fragment.append(...this.nodes(children));
    return fragment;
  }
}
