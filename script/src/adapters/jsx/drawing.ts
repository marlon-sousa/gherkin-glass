// Adapter: markup drawn into a document, every element made with `createElement` and every text with `createTextNode`, so that no text of a feature is ever read as markup.

import type { Child } from "./jsx_runtime.ts";

/** The nodes `child` describes, made in `document`; a node it holds is moved, not copied. */
export function drawn(document: Document, child: Child): Node[] {
  if (child === null || child === false) {
    return [];
  }
  if (typeof child === "string") {
    return [document.createTextNode(child)];
  }
  if (isList(child)) {
    return child.flatMap((inner) => drawn(document, inner));
  }
  if ("nodeType" in child) {
    return [child];
  }
  if (child.tag === null) {
    return drawn(document, child.children);
  }
  const element = document.createElement(child.tag);
  for (const [name, value] of Object.entries(child.attributes)) {
    if (value === true) {
      element.setAttribute(name, "");
    } else if (typeof value === "string") {
      element.setAttribute(name, value);
    }
  }
  element.append(...drawn(document, child.children));
  return [element];
}

/** `Array.isArray` narrows a readonly list to `any[]`. */
function isList(child: Child): child is readonly Child[] {
  return Array.isArray(child);
}
