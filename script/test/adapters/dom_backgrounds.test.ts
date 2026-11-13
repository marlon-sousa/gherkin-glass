// The adapter of src/adapters/dom_backgrounds.ts, on pages gherkin-glass wrote:
// tests/fixtures/backgrounds/expected and expected-linked, with the service behind it.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomBackgrounds } from "../../src/adapters/dom_backgrounds.ts";
import { DomPage } from "../../src/adapters/dom_page.ts";
import { startCollapsing } from "../../src/services/collapsing.ts";
import type { Memory } from "../../src/ports/memory.ts";

const SITES = new URL("../../../tests/fixtures/backgrounds/", import.meta.url);

function open(site: string, page: string, memory: Memory = keeping()): Window {
  const window = new Window({ url: `https://example.com/site/${page}` });
  window.document.write(readFileSync(new URL(`${site}/${page}`, SITES), "utf8"));
  const document = window.document as unknown as Document;
  const backgrounds = DomBackgrounds.of(document);
  startCollapsing(new DomPage(document), memory, window.location.pathname, backgrounds && { backgrounds, siteRoot: "/site/" });
  return window;
}

function keeping(kept = new Map<string, string>()): Memory {
  return { get: (key) => kept.get(key) ?? null, set: (key, value) => void kept.set(key, value) };
}

const checkbox = (window: Window) => window.document.querySelector("input.ghg-inline-backgrounds") as unknown as HTMLInputElement;
const shown = (window: Window, selector: string): boolean[] =>
  [...window.document.querySelectorAll(`.ghg-from-background ${selector}`)].map((element) => !(element as unknown as HTMLElement).hidden);

/** For every background a scenario starts with: whether its steps show, and whether its link does. */
function forms(window: Window): [boolean, boolean][] {
  const steps = shown(window, "> .ghg-inline");
  const links = shown(window, ".ghg-linked");
  assert.equal(steps.length, links.length);
  return steps.map((step, index) => [step, links[index]!]);
}

test("a page written inline shows every background's steps, and its checkbox is checked", () => {
  const window = open("expected", "both.html");
  assert.equal(checkbox(window).checked, true);
  assert.equal(forms(window).length, 4);
  assert.ok(forms(window).every(([steps, link]) => steps && !link));
});

test("a page written linked shows every link, and its checkbox is not checked", () => {
  const window = open("expected-linked", "both.html");
  assert.equal(checkbox(window).checked, false);
  assert.ok(forms(window).every(([steps, link]) => !steps && link));
});

test("unchecking the checkbox links the backgrounds, checking it shows them again, and the focus stays on it", () => {
  const window = open("expected", "both.html");
  const box = checkbox(window);
  box.focus();
  box.click();
  assert.equal(box.checked, false);
  assert.ok(forms(window).every(([steps, link]) => !steps && link));
  box.click();
  assert.ok(forms(window).every(([steps, link]) => steps && !link));
  assert.equal(window.document.activeElement, box as unknown);
});

test("the reader's choice is remembered for every page of the site, and wins over the form a page was written in", () => {
  const kept = new Map<string, string>();
  checkbox(open("expected", "both.html", keeping(kept))).click();
  const other = open("expected", "regra.html", keeping(kept));
  assert.equal(checkbox(other).checked, false);
  assert.ok(forms(other).every(([steps, link]) => !steps && link));
});

test("a checkbox a browser restored is set back to the form the page shows", () => {
  const window = new Window({ url: "https://example.com/site/both.html" });
  window.document.write(readFileSync(new URL("expected/both.html", SITES), "utf8"));
  const document = window.document as unknown as Document;
  const box = checkbox(window);
  box.checked = false;
  startCollapsing(new DomPage(document), keeping(), "/site/both.html", { backgrounds: DomBackgrounds.of(document)!, siteRoot: "/site/" });
  assert.equal(box.checked, true);
  assert.ok(forms(window).every(([steps, link]) => steps && !link));
});

test("a page with a background no scenario starts with has the checkbox disabled, and a page with no background has none", () => {
  for (const page of ["no-steps.html", "no-scenarios.html"]) {
    assert.equal(checkbox(open("expected", page)).disabled, true, page);
  }
  assert.equal(checkbox(open("expected", "both.html")).disabled, false);
  const window = open("expected", "none.html");
  assert.equal(DomBackgrounds.of(window.document as unknown as Document), null);
});

test("each list of a background's steps is named by the words that introduce it", () => {
  const window = open("expected", "both.html");
  for (const list of window.document.querySelectorAll(".ghg-from-background > ol")) {
    const introduction = window.document.getElementById(list.getAttribute("aria-labelledby")!)!;
    assert.equal(introduction.parentElement, list.parentElement);
    assert.match(introduction.textContent!, /^From the (rule's )?background:/);
  }
});

test("a page shown again from the browser's cache takes the choice made on another page meanwhile", () => {
  const kept = new Map<string, string>();
  const window = open("expected", "both.html", keeping(kept));
  checkbox(window).click();
  kept.set("gherkin-glass:/site/:backgrounds", "inline");
  const event = new window.Event("pageshow") as unknown as { persisted: boolean };
  Object.defineProperty(event, "persisted", { value: true });
  window.dispatchEvent(event as unknown as InstanceType<typeof window.Event>);
  assert.equal(checkbox(window).checked, true);
  assert.ok(forms(window).every(([steps, link]) => steps && !link));
});

test("following a link to a background expands it", () => {
  const kept = new Map<string, string>([["gherkin-glass:/site/both.html#background", "collapsed"]]);
  const window = open("expected-linked", "both.html", keeping(kept));
  const heading = window.document.querySelector("#background > button")!;
  assert.equal(heading.getAttribute("aria-expanded"), "false");
  (window.document.querySelector(".ghg-linked a[href='#background']") as unknown as HTMLElement).click();
  assert.equal(heading.getAttribute("aria-expanded"), "true");
});

const backgroundsExpanded = (window: Window): boolean[] =>
  [...window.document.querySelectorAll(".ghg-background > :is(h2, h3) > button")].map((button) => button.getAttribute("aria-expanded") === "true");

test("the backgrounds whose steps show inside each scenario are collapsed, and the checkbox collapses or expands them all", () => {
  const window = open("expected", "both.html");
  assert.deepEqual(backgroundsExpanded(window), [false, false]);
  const box = checkbox(window);
  box.click();
  assert.deepEqual(backgroundsExpanded(window), [true, true]);
  (window.document.querySelector("#background > button") as unknown as HTMLElement).click();
  assert.deepEqual(backgroundsExpanded(window), [false, true]);
  box.click();
  assert.deepEqual(backgroundsExpanded(window), [false, false]);
  box.click();
  assert.deepEqual(backgroundsExpanded(window), [true, true]);
});

test("a page written linked collapses its backgrounds when the inline form was chosen on another page", () => {
  const kept = new Map<string, string>([["gherkin-glass:/site/:backgrounds", "inline"]]);
  assert.deepEqual(backgroundsExpanded(open("expected-linked", "both.html", keeping(kept))), [false, false]);
  assert.deepEqual(backgroundsExpanded(open("expected-linked", "both.html")), [true, true]);
});

test("a background whose steps no scenario starts with is not collapsed by the inline form", () => {
  for (const page of ["no-steps.html", "no-scenarios.html"]) {
    assert.deepEqual(backgroundsExpanded(open("expected", page)), [true], page);
  }
});
