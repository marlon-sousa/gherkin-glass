// Adapter: the JSX runtime the pages are written in, which esbuild's automatic transform calls, describing each element as data that `drawing.ts` draws, and the tags and attributes tsc lets the pages use.

/** An attribute's value: text, `true` for one with no value, or `null` and `false` for none. */
export type AttributeValue = string | boolean | null;

/** What an element may hold: markup, a node of the document moved into it, text made into a text node, a list of them, or `null` and `false` for nothing. */
export type Child = Markup | Node | string | null | false | readonly Child[];

export interface Markup {
  /** Null for a fragment, whose children are drawn in its place. */
  readonly tag: string | null;
  /** In the order the element's attributes are set. */
  readonly attributes: Readonly<Record<string, AttributeValue>>;
  readonly children: Child;
}

interface Props {
  readonly children?: Child;
  readonly [name: string]: AttributeValue | Child | undefined;
}

type Component = (props: Props) => Markup;

export function Fragment(props: Props): Markup {
  return { tag: null, attributes: {}, children: props.children ?? null };
}

export function jsx(type: string | Component, props: Props): Markup {
  if (typeof type !== "string") {
    return type(props);
  }
  const attributes: Record<string, AttributeValue> = {};
  for (const [name, value] of Object.entries(props)) {
    if (name !== "children" && (typeof value === "string" || typeof value === "boolean")) {
      attributes[name] = value;
    }
  }
  return { tag: type, attributes, children: props.children ?? null };
}

/** Several children, which esbuild 0.28.2 calls for an element written with more than one. */
export const jsxs = jsx;

interface Attributes {
  readonly id?: string;
  readonly class?: string;
  readonly lang?: string | null;
  readonly hidden?: boolean;
  readonly role?: string;
  readonly tabindex?: string;
  readonly "aria-controls"?: string;
  readonly "aria-current"?: "page" | false;
  readonly "aria-describedby"?: string;
  readonly "aria-disabled"?: "true";
  readonly "aria-expanded"?: "true" | "false" | null;
  readonly "aria-haspopup"?: "dialog";
  readonly "aria-hidden"?: "true";
  readonly "aria-label"?: string;
  readonly "aria-labelledby"?: string;
  readonly "aria-owns"?: string | null;
  readonly "aria-selected"?: "true" | "false";
  /** tsc 7.0.2 checks the value of an attribute whose name holds a hyphen only when the attribute is named, not when an index signature such as this one gives it. */
  readonly [data: `data-${string}`]: string | null;
}

interface Holding extends Attributes {
  readonly children?: Child;
}

interface Empty extends Attributes {
  readonly children?: never;
}

export declare namespace JSX {
  type Element = Markup;

  interface ElementChildrenAttribute {
    children: unknown;
  }

  /** Only the tags the pages draw, each with only the attributes they set. */
  interface IntrinsicElements {
    a: Holding & { readonly href: string };
    bdi: Holding;
    br: Empty;
    button: Holding & { readonly type?: "button" };
    dialog: Holding;
    div: Holding;
    figcaption: Holding;
    figure: Holding;
    form: Holding & { readonly method: "dialog" };
    h1: Holding;
    h2: Holding;
    h3: Holding;
    h4: Holding;
    h5: Holding;
    h6: Holding;
    input: Empty & { readonly type: "checkbox"; readonly autocomplete: "off"; readonly checked: boolean; readonly disabled: boolean };
    label: Holding;
    li: Holding;
    mark: Holding;
    nav: Holding;
    ol: Holding;
    p: Holding;
    pre: Holding;
    span: Holding;
    table: Holding;
    tbody: Holding;
    td: Holding;
    th: Holding & { readonly scope: "col" };
    thead: Holding;
    tr: Holding;
    ul: Holding;
  }
}
