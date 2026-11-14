// docs/BEHAVIOUR.md, "The folder tree", rules 87 and 88, checked in Chromium
// with Playwright: tests/fixtures/pages is built and served, and a feature
// page is loaded on a narrow window, where the button shows and hides the tree
// and Escape in the tree hides it again, with the focus on the button, and on
// a wide window resized to narrow and back without a reload, where the
// checkbox and the button change places at once, and the focus, when it was
// in a tree the resize hides, moves to the control shown. The page is held to
// the rules of "Accessibility" with the tree shown on a narrow window, in the
// light and the dark colour scheme.
//
// Run from the repository's root, after cargo build and npm ci in script/,
// with Playwright's Chromium installed:
//   node tests/browser/narrow.mjs
// cargo xtask check-browser does all of it. GHG names another ghg binary than
// target/debug/ghg.

import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import assert from "node:assert/strict";
import { axeSource, chromium, ghg, serve, stop, violations } from "./support.mjs";

const NARROW = { width: 390, height: 844 };
const WIDE = { width: 1366, height: 850 };
const PAGE = "billing/refunds.html";

// What the reader meets: which control shows, the button's state, whether the
// tree shows, and where the focus is.
const seen = (page) =>
  page.evaluate(() => {
    const shown = (element) => element !== null && element.checkVisibility();
    const button = document.querySelector("button.ghg-tree-button");
    return {
      checkbox: shown(document.querySelector("input.ghg-show-tree")),
      button: shown(button),
      expanded: button?.getAttribute("aria-expanded"),
      tree: shown(document.querySelector("nav.ghg-tree")),
      focus:
        document.activeElement === button
          ? "button"
          : document.activeElement?.matches("input.ghg-show-tree")
            ? "checkbox"
            : document.activeElement?.getAttribute("role") ?? document.activeElement?.tagName,
    };
  });

const treeBuilt = (page) => page.waitForFunction(() => document.querySelector('nav.ghg-tree [role="tree"]') !== null);

const output = mkdtempSync(join(tmpdir(), "ghg-narrow-"));
const browser = await chromium.launch();
const problems = [];
const expect = (what, found, wanted) => {
  for (const [key, value] of Object.entries(wanted)) {
    if (found[key] !== value) problems.push(`${what}: ${key} is ${JSON.stringify(found[key])}, not ${JSON.stringify(value)}`);
  }
};
let running;
try {
  const run = spawnSync(ghg, ["build", "--output", output], { cwd: resolve("tests/fixtures/pages"), encoding: "utf8" });
  assert.equal(run.status, 0, `ghg build in tests/fixtures/pages: ${run.stderr}`);
  running = await serve(["--no-build", "--output", output, "--port", "0"], output);
  const address = new URL(PAGE, running.address).href;
  for (const scheme of ["light", "dark"]) {
    const context = await browser.newContext({ colorScheme: scheme, viewport: NARROW });
    await context.addInitScript({ path: axeSource });
    const page = await context.newPage();
    await page.goto(address);
    await treeBuilt(page);
    expect(`${scheme}, loaded narrow`, await seen(page), { checkbox: false, button: true, expanded: "false", tree: false });
    await page.locator("button.ghg-tree-button").focus();
    await page.keyboard.press("Enter");
    expect(`${scheme}, the button pressed`, await seen(page), { button: true, expanded: "true", tree: true });
    for (const line of await violations(page)) problems.push(`${scheme}, narrow with the tree shown: ${line}`);
    // The tree comes after the options bar for the keyboard, as on the screen:
    // after the button, Tab passes the theme's radio buttons, then reaches it.
    await page.keyboard.press("Tab");
    expect(`${scheme}, Tab after the button`, await seen(page), { focus: "INPUT" });
    await page.keyboard.press("Tab");
    expect(`${scheme}, Tab after the theme`, await seen(page), { focus: "treeitem" });
    await page.keyboard.press("Escape");
    expect(`${scheme}, Escape in the tree`, await seen(page), { expanded: "false", tree: false, focus: "button" });
    await context.close();
  }
  const context = await browser.newContext({ viewport: WIDE });
  const page = await context.newPage();
  await page.goto(address);
  await treeBuilt(page);
  expect("loaded wide", await seen(page), { checkbox: true, button: false, tree: true });
  await page.setViewportSize(NARROW);
  await page.waitForFunction(() => document.querySelector("nav.ghg-tree").hidden);
  expect("resized to narrow", await seen(page), { checkbox: false, button: true, expanded: "false", tree: false });
  const top = await page.evaluate(() => document.querySelector("h1").getBoundingClientRect().top);
  if (top > NARROW.height / 2) problems.push(`resized to narrow: the h1 is ${top} pixels from the top`);
  await page.locator("button.ghg-tree-button").click();
  await page.setViewportSize(WIDE);
  await page.waitForFunction(() => document.querySelector("button.ghg-tree-button").getAttribute("aria-expanded") === "false");
  expect("resized to wide again", await seen(page), { checkbox: true, button: false, tree: true });
  // A resize that hides the tree with the focus in it moves the focus to the
  // control it shows: the button, then, with the box unchecked, the checkbox.
  await page.locator('nav.ghg-tree [role="treeitem"][tabindex="0"]').focus();
  await page.setViewportSize(NARROW);
  await page.waitForFunction(() => document.querySelector("nav.ghg-tree").hidden);
  expect("resized to narrow with the focus in the tree", await seen(page), { tree: false, focus: "button" });
  await page.setViewportSize(WIDE);
  await page.locator("input.ghg-show-tree").uncheck();
  await page.setViewportSize(NARROW);
  await page.locator("button.ghg-tree-button").click();
  await page.locator('nav.ghg-tree [role="treeitem"][tabindex="0"]').focus();
  await page.setViewportSize(WIDE);
  await page.waitForFunction(() => document.querySelector("nav.ghg-tree").hidden);
  expect("resized to wide with the focus in the tree", await seen(page), { checkbox: true, tree: false, focus: "checkbox" });
  await context.close();
} finally {
  if (running) await stop(running.server);
  await browser.close();
  rmSync(output, { recursive: true, force: true });
}
if (problems.length > 0) {
  console.error(problems.join("\n"));
  process.exit(1);
}
console.log("The folder tree follows the window's width, with its checkbox and its button.");
