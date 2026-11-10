// The adapters of src/adapters/dom_tree.ts and src/adapters/fetched_tree.ts, on a page gherkin-glass wrote,
// tests/fixtures/pages/expected/billing/refunds.html, with its site's tree.json: docs/BEHAVIOUR.md, rules 87 and 89 to 93.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomTree, DomTreeSwitch } from "../../src/adapters/dom_tree.ts";
import { FetchedTree } from "../../src/adapters/fetched_tree.ts";
import { NARROW, readTree } from "../../src/policies/tree_state.ts";
import { startFolderTree } from "../../src/services/folder_tree.ts";
import type { Memory } from "../../src/ports/memory.ts";

const SITE = new URL("../../../tests/fixtures/pages/expected/", import.meta.url);
const DATA = readFileSync(new URL("assets/tree.json", SITE), "utf8");

async function open(kept = new Map<string, string>()): Promise<Window> {
  const window = new Window({ url: "https://example.com/site/billing/refunds.html" });
  window.document.write(readFileSync(new URL("billing/refunds.html", SITE), "utf8"));
  const document = window.document as unknown as Document;
  const memory: Memory = { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
  const source = { load: () => Promise.resolve(readTree(JSON.parse(DATA))) };
  await startFolderTree(source, new DomTreeSwitch(document), new DomTree(document, new URL("https://example.com/site/"), "billing/refunds.html"), memory, {
    sitePath: "/site/",
    page: "billing/refunds.html",
    narrow: false,
  });
  return window;
}

const items = (window: Window) => [...window.document.querySelectorAll('[role="treeitem"]')] as unknown as HTMLAnchorElement[];
const item = (window: Window, text: string) => items(window).find((link) => link.textContent === text)!;
const visible = (window: Window) => items(window).filter((link) => !link.closest("[hidden]")).map((link) => link.textContent);
const focused = (window: Window) => window.document.activeElement?.textContent;
const press = (window: Window, key: string) =>
  window.document.activeElement!.dispatchEvent(new window.KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));

test("the tree is built in its landmark, named as it, with every item a link relative to the page", async () => {
  const window = await open();
  const tree = window.document.querySelector('nav.ghg-tree > ul[role="tree"]')!;
  assert.equal(tree.getAttribute("aria-label"), "Folder tree");
  const home = items(window)[0]!;
  assert.equal(home.textContent, "Banking & <more>");
  assert.equal(home.getAttribute("href"), "../index.html");
  assert.equal(item(window, "Example: Store credit").getAttribute("href"), "../billing/refunds.html#store-credit");
  assert.equal(item(window, "Feature: Closed accounts").getAttribute("href"), "../billing/archive/closed%20%231.html");
  // A feature's item, with the items inside it, has its page's lang.
  assert.equal(item(window, "Feature: Refunds & <returns>").parentElement!.getAttribute("lang"), "en");
  assert.equal(home.parentElement!.hasAttribute("lang"), false);
});

test("the current page is marked, and it and the items it is inside are expanded, everything else collapsed", async () => {
  const window = await open();
  const current = items(window).filter((link) => link.getAttribute("aria-current") === "page");
  assert.deepEqual(current.map((link) => link.textContent), ["Feature: Refunds & <returns>"]);
  assert.equal(item(window, "billing").getAttribute("aria-expanded"), "true");
  assert.equal(item(window, "Feature: Refunds & <returns>").getAttribute("aria-expanded"), "true");
  assert.equal(item(window, "archive").getAttribute("aria-expanded"), "false");
  assert.equal(item(window, "reports").getAttribute("aria-expanded"), "false");
  assert.equal(item(window, "Home") ?? null, null);
  // A collapsed item's group is hidden, and an item with nothing inside it has no marker.
  assert.equal(visible(window).includes("Feature: Closed accounts"), false);
  assert.equal(items(window)[0]!.querySelector(".ghg-marker"), null);
  const owned = item(window, "billing").getAttribute("aria-owns")!;
  assert.equal(window.document.getElementById(owned)!.getAttribute("role"), "group");
  assert.match(owned, /_/);
});

test("the tree is one Tab stop, on the current page's item", async () => {
  const window = await open();
  const stops = items(window).filter((link) => link.tabIndex === 0);
  assert.deepEqual(stops.map((link) => link.textContent), ["Feature: Refunds & <returns>"]);
});

test("the arrow keys, Home and End move through the visible items and expand and collapse them", async () => {
  const kept = new Map<string, string>();
  const window = await open(kept);
  const billing = item(window, "billing");
  billing.focus();
  press(window, "ArrowUp");
  assert.equal(focused(window), "Banking & <more>");
  press(window, "ArrowUp");
  assert.equal(focused(window), "Banking & <more>");
  press(window, "ArrowDown");
  press(window, "ArrowDown");
  assert.equal(focused(window), "archive");
  press(window, "ArrowRight");
  assert.equal(item(window, "archive").getAttribute("aria-expanded"), "true");
  assert.equal(kept.get("gherkin-glass:/site/ tree billing/archive/index.html"), "expanded");
  assert.equal(focused(window), "archive");
  press(window, "ArrowRight");
  assert.equal(focused(window), "Feature: Closed accounts");
  press(window, "ArrowLeft");
  assert.equal(focused(window), "archive");
  press(window, "ArrowLeft");
  assert.equal(item(window, "archive").getAttribute("aria-expanded"), "false");
  assert.equal(kept.get("gherkin-glass:/site/ tree billing/archive/index.html"), "collapsed");
  press(window, "ArrowLeft");
  assert.equal(focused(window), "billing");
  press(window, "End");
  assert.equal(focused(window), visible(window).at(-1));
  press(window, "Home");
  assert.equal(focused(window), "Banking & <more>");
  // The focused item is the one Tab stop.
  assert.deepEqual(items(window).filter((link) => link.tabIndex === 0).map((link) => link.textContent), ["Banking & <more>"]);
});

test("a letter moves to the next visible item starting with it, and * expands the items beside", async () => {
  const window = await open();
  items(window)[0]!.focus();
  press(window, "R");
  assert.equal(focused(window), "Rule: Refunds after thirty days");
  press(window, "r");
  assert.equal(focused(window), "Rule");
  press(window, "r");
  assert.equal(focused(window), "reports");
  // From the last match, the search starts again from the first item.
  press(window, "b");
  assert.equal(focused(window), "Banking & <more>");
  press(window, "b");
  assert.equal(focused(window), "billing");
  press(window, "*");
  for (const sibling of ["billing", "reports", "stories"]) {
    assert.equal(item(window, sibling).getAttribute("aria-expanded"), "true", sibling);
  }
  assert.equal(item(window, "archive").getAttribute("aria-expanded"), "false");
});

test("pressing a marker expands or collapses its item, and Space follows the link", async () => {
  const window = await open();
  const reports = item(window, "reports");
  const followed: string[] = [];
  window.document.addEventListener("click", (event) => {
    if (!event.defaultPrevented) {
      followed.push((event.target as unknown as Element).closest("a")!.textContent ?? "");
      event.preventDefault();
    }
  });
  (reports.querySelector(".ghg-marker") as unknown as HTMLElement).click();
  assert.equal(reports.getAttribute("aria-expanded"), "true");
  assert.deepEqual(followed, []);
  reports.focus();
  press(window, " ");
  assert.deepEqual(followed, ["reports"]);
});

test("the checkbox shows and hides the tree, remembered for the whole site", async () => {
  const kept = new Map<string, string>();
  const window = await open(kept);
  const checkbox = window.document.querySelector("#ghg_show_tree") as unknown as HTMLInputElement;
  const landmark = window.document.querySelector("#ghg_tree")!;
  assert.equal(checkbox.checked, true);
  assert.equal(landmark.hasAttribute("hidden"), false);
  checkbox.click();
  assert.equal(landmark.hasAttribute("hidden"), true);
  assert.equal(kept.get("gherkin-glass:/site/ tree"), "hidden");
  const again = await open(kept);
  assert.equal((again.document.querySelector("#ghg_show_tree") as unknown as HTMLInputElement).checked, false);
  assert.equal(again.document.querySelector("#ghg_tree")!.hasAttribute("hidden"), true);
});

test("data that cannot be fetched, or is no tree, gives nothing to build", async () => {
  const answer = (status: number, body: string) => () => Promise.resolve(new Response(body, { status }));
  const at = new URL("https://example.com/site/assets/tree.json");
  assert.equal(await new FetchedTree(at, answer(404, "") as typeof fetch).load(), null);
  assert.equal(await new FetchedTree(at, answer(200, "{}") as typeof fetch).load(), null);
  assert.equal(await new FetchedTree(at, answer(200, "not json") as typeof fetch).load(), null);
  assert.equal(await new FetchedTree(at, (() => Promise.reject(new Error("offline"))) as typeof fetch).load(), null);
  assert.equal((await new FetchedTree(at, answer(200, DATA) as typeof fetch).load())?.length, 7);
});

test("a page that is not below the site's root links the items from the root itself", async () => {
  const window = new Window({ url: "https://example.com/elsewhere/deep/page.html" });
  window.document.write(readFileSync(new URL("billing/refunds.html", SITE), "utf8"));
  const tree = new DomTree(window.document as unknown as Document, new URL("https://example.com/site/"), null);
  tree.build(readTree(JSON.parse(DATA))!, null, () => false);
  assert.equal(items(window)[1]!.getAttribute("href"), "https://example.com/site/billing/index.html");
  // Without a current page, the first item is the Tab stop.
  assert.equal(items(window)[0]!.tabIndex, 0);
});

test("data that cannot be loaded leaves the checkbox disabled and unchecked and the tree hidden and empty", async () => {
  const window = new Window({ url: "https://example.com/site/billing/refunds.html" });
  window.document.write(readFileSync(new URL("billing/refunds.html", SITE), "utf8"));
  const document = window.document as unknown as Document;
  const memory: Memory = { get: () => null, set: () => {} };
  await startFolderTree({ load: () => Promise.resolve(null) }, new DomTreeSwitch(document), new DomTree(document, new URL("https://example.com/site/"), "billing/refunds.html"), memory, {
    sitePath: "/site/",
    page: "billing/refunds.html",
    narrow: false,
  });
  const checkbox = window.document.querySelector("#ghg_show_tree") as unknown as HTMLInputElement;
  assert.equal(checkbox.disabled, true);
  assert.equal(checkbox.checked, false);
  const landmark = window.document.querySelector("#ghg_tree")!;
  assert.equal(landmark.hasAttribute("hidden"), true);
  assert.equal(landmark.children.length, 0);
});

test("the window the script calls narrow is the one the stylesheet stacks the tree at", () => {
  const css = readFileSync(new URL("../../../assets/gherkin-glass.css", import.meta.url), "utf8");
  assert.ok(css.includes(`@media ${NARROW} {`), NARROW);
});
