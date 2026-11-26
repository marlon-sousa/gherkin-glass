// Policy: what follows the `#` of a shell's address, read into the page and the anchor it names, and written back from them.

/** A page of a shell, by its kind and its name. */
export type PageName =
  | { readonly kind: "home" }
  /** `folder` is the folder's path below the input folder, ending in `/`. */
  | { readonly kind: "folder"; readonly folder: string }
  | { readonly kind: "feature"; readonly page: string }
  | { readonly kind: "tags" }
  /** `tag` is the tag page's name, without `tags/`. */
  | { readonly kind: "tag"; readonly tag: string }
  | { readonly kind: "steps" }
  /** `words` are empty text when the address has none. */
  | { readonly kind: "search"; readonly words: string };

/** What an address names: a page, and the anchor or `id` on it, or, after `##`, an element of the page shown. */
export type Target =
  | { readonly kind: "page"; readonly page: PageName; readonly anchor: string | null }
  | { readonly kind: "element"; readonly id: string };

const TAGS = "tags/";
const STEPS = "steps";
const SEARCH = "search";

/** Every byte but these is percent-encoded, as ghg encodes a path. */
const KEPT = /[A-Za-z0-9\-._~/]/;

/** `hash` is the address's fragment with its `#`, or empty text. */
export function readTarget(hash: string): Target {
  const rest = hash.startsWith("#") ? hash.slice(1) : hash;
  if (rest.startsWith("#")) {
    return { kind: "element", id: decoded(rest.slice(1)) };
  }
  const split = rest.indexOf("#");
  const name = split < 0 ? rest : rest.slice(0, split);
  const anchor = split < 0 || split === rest.length - 1 ? null : decoded(rest.slice(split + 1));
  return { kind: "page", page: pageNamed(name), anchor };
}

/** The address, from `#`, of `page`, and of `anchor` on it when one is given, as every link between pages of one shell is written. */
export function targetAddress(page: PageName, anchor: string | null = null): string {
  const named = `#${encodedName(page)}`;
  return anchor === null ? named : `${named}#${encoded(anchor)}`;
}

/** The address, from `#`, of an element of the page shown, such as `##ghg_main`. */
export function elementAddress(id: string): string {
  return `##${encoded(id)}`;
}

/** The page name `raw`, as the address holds it, encoded. */
function pageNamed(raw: string): PageName {
  if (raw === SEARCH || raw.startsWith(`${SEARCH}?`)) {
    return { kind: "search", words: new URLSearchParams(raw.slice(SEARCH.length + 1)).get("q") ?? "" };
  }
  return pageByName(decoded(raw));
}

/** The page of the store's name for it, such as a descriptor's `page`, decoded. */
export function pageByName(name: string): PageName {
  if (name === "") {
    return { kind: "home" };
  }
  if (name === TAGS) {
    return { kind: "tags" };
  }
  if (name.startsWith(TAGS)) {
    return { kind: "tag", tag: name.slice(TAGS.length) };
  }
  if (name === STEPS) {
    return { kind: "steps" };
  }
  return name.endsWith("/") ? { kind: "folder", folder: name } : { kind: "feature", page: name };
}

function encodedName(page: PageName): string {
  switch (page.kind) {
    case "home":
      return "";
    case "folder":
      return encoded(page.folder);
    case "feature":
      return encoded(page.page);
    case "tags":
      return TAGS;
    case "tag":
      return encoded(`${TAGS}${page.tag}`);
    case "steps":
      return STEPS;
    case "search":
      return page.words === "" ? SEARCH : `${SEARCH}?${new URLSearchParams({ q: page.words }).toString()}`;
  }
}

/** `text` with each byte of UTF-8 outside the kept characters written `%` and two capital hexadecimal digits. */
export function encoded(text: string): string {
  return [...text].map(encodedCharacter).join("");
}

function encodedCharacter(character: string): string {
  if (KEPT.test(character)) {
    return character;
  }
  return [...new TextEncoder().encode(character)].map(percentByte).join("");
}

function percentByte(byte: number): string {
  return `%${byte.toString(16).toUpperCase().padStart(2, "0")}`;
}

/** Text that is not percent-encoded UTF-8 names itself. */
function decoded(text: string): string {
  try {
    return decodeURIComponent(text);
  } catch {
    return text;
  }
}
