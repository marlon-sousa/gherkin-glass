// The adapter of src/adapters/dom_backgrounds.ts, on pages gherkin-glass wrote:
// tests/fixtures/backgrounds/expected and expected-unchecked, with the service behind it.

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

/** Whether each background inside a scenario is expanded, in the order of the page. */
const inside = (window: Window): boolean[] =>
  [...window.document.querySelectorAll(".ghg-background-in-scenario > :is(h3, h4) > button")].map((button) => button.getAttribute("aria-expanded") === "true");

test("a page written with the box checked expands every background inside a scenario", () => {
  const window = open("expected", "both.html");
  assert.equal(checkbox(window).checked, true);
  assert.deepEqual(inside(window), [true, true, true, true, true]);
});

test("a page written with the box unchecked collapses every background inside a scenario", () => {
  const window = open("expected-unchecked", "both.html");
  assert.equal(checkbox(window).checked, false);
  assert.deepEqual(inside(window), [false, false, false, false, false]);
});

test("unchecking the checkbox collapses the backgrounds inside the scenarios, checking it expands them again, and the focus stays on it", () => {
  const window = open("expected", "both.html");
  const box = checkbox(window);
  box.focus();
  box.click();
  assert.equal(box.checked, false);
  assert.deepEqual(inside(window), [false, false, false, false, false]);
  box.click();
  assert.deepEqual(inside(window), [true, true, true, true, true]);
  assert.equal(window.document.activeElement, box as unknown);
});

test("the reader's choice is remembered for every page of the site, and wins over the state a page was written with", () => {
  const kept = new Map<string, string>();
  checkbox(open("expected", "both.html", keeping(kept))).click();
  const other = open("expected", "regra.html", keeping(kept));
  assert.equal(checkbox(other).checked, false);
  assert.deepEqual(inside(other), [false]);
});

test("a checkbox a browser restored is set back to the state the page shows", () => {
  const window = new Window({ url: "https://example.com/site/both.html" });
  window.document.write(readFileSync(new URL("expected/both.html", SITES), "utf8"));
  const document = window.document as unknown as Document;
  const box = checkbox(window);
  box.checked = false;
  startCollapsing(new DomPage(document), keeping(), "/site/both.html", { backgrounds: DomBackgrounds.of(document)!, siteRoot: "/site/" });
  assert.equal(box.checked, true);
  assert.deepEqual(inside(window), [true, true, true, true, true]);
});

test("a page with a background no scenario starts with has the checkbox disabled, and a page with no background has none", () => {
  for (const page of ["no-steps.html", "no-scenarios.html"]) {
    assert.equal(checkbox(open("expected", page)).disabled, true, page);
  }
  assert.equal(checkbox(open("expected", "both.html")).disabled, false);
  const window = open("expected", "none.html");
  assert.equal(DomBackgrounds.of(window.document as unknown as Document), null);
});

test("each background inside a scenario is a block one level below the scenario, headed by where its steps come from", () => {
  const window = open("expected", "both.html");
  const headings = [...window.document.querySelectorAll(".ghg-background-in-scenario > :is(h3, h4)")].map((heading) => {
    const scenario = heading.parentElement!.parentElement!.closest(".ghg-block")!.querySelector(":scope > :is(h2, h3)")!;
    return [heading.id, Number(heading.tagName.slice(1)) - Number(scenario.tagName.slice(1)), heading.textContent];
  });
  assert.deepEqual(headings, [
    ["outside-any-rule_background_1", 1, "From the background"],
    ["move-amount_background_1", 1, "From the background"],
    ["move-amount_background_2", 1, "From the rule's background"],
    ["only-the-features-background_background_1", 1, "From the background"],
    ["nothing-of-its-own_background_1", 1, "From the background"],
  ]);
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
  assert.deepEqual(inside(window), [true, true, true, true, true]);
});

test("following a link to a background expands it", () => {
  const kept = new Map<string, string>([["gherkin-glass:/site/both.html#background", "collapsed"]]);
  const window = open("expected-unchecked", "both.html", keeping(kept));
  const heading = window.document.querySelector("#background > button")!;
  assert.equal(heading.getAttribute("aria-expanded"), "false");
  const link = window.document.createElement("a");
  link.setAttribute("href", "#background");
  window.document.body.appendChild(link);
  link.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }));
  assert.equal(heading.getAttribute("aria-expanded"), "true");
});

const backgroundsExpanded = (window: Window): boolean[] =>
  [...window.document.querySelectorAll(".ghg-background > :is(h2, h3) > button")].map((button) => button.getAttribute("aria-expanded") === "true");

test("the backgrounds in their own place whose steps a scenario holds are collapsed, and the checkbox collapses or expands them all", () => {
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

test("a page written unchecked collapses its backgrounds in their own place when the box was checked on another page", () => {
  const kept = new Map<string, string>([["gherkin-glass:/site/:backgrounds", "inline"]]);
  assert.deepEqual(backgroundsExpanded(open("expected-unchecked", "both.html", keeping(kept))), [false, false]);
  assert.deepEqual(backgroundsExpanded(open("expected-unchecked", "both.html")), [true, true]);
});

test("a background whose steps no scenario starts with is not collapsed by the checked box", () => {
  for (const page of ["no-steps.html", "no-scenarios.html"]) {
    assert.deepEqual(backgroundsExpanded(open("expected", page)), [true], page);
  }
});
