// docs/BEHAVIOUR.md, "Reloading", rules 71 and 72, checked in Chromium with
// Playwright: ghg serve --watch serves a copy of tests/fixtures/serve/reload,
// a feature changes, and the page must reload at the reader's heading.
//
// Run from the repository's root, after cargo build, with Playwright
// installed where Node finds it:
//   node tests/browser/reload.mjs
// GHG names another ghg binary than target/debug/ghg.

import { spawn } from "node:child_process";
import { appendFileSync, cpSync, mkdtempSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createInterface } from "node:readline";
import assert from "node:assert/strict";

const require = createRequire(import.meta.url);
const { chromium } = require("playwright");

const ghg = resolve(process.env.GHG ?? "target/debug/ghg");
const folder = mkdtempSync(join(tmpdir(), "ghg-reload-"));
cpSync("tests/fixtures/serve/reload", folder, { recursive: true });
const feature = join(folder, "features/many-scenarios.feature");

const server = spawn(ghg, ["serve", "--watch", "--port", "0"], { cwd: folder });
const firstLine = await new Promise((done) =>
  createInterface({ input: server.stdout }).once("line", done),
);
const address = /^The site is served at (\S+)\. /.exec(firstLine)?.[1];
assert.ok(address, firstLine);

const browser = await chromium.launch();
let build = 1;

try {
  const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
  const heading = (n) => `scenario-number-${n}`;

  // Changes a feature, then waits for the page of the next version.
  async function rebuild() {
    build += 1;
    appendFileSync(feature, "\n");
    await page.waitForFunction(
      (number) =>
        document.querySelector('script[src*="/__ghg/reload.js"]')?.src.endsWith(`build=${number}`),
      build,
      { timeout: 15000 },
    );
    await page.waitForLoadState("load");
    await page.waitForTimeout(200);
  }

  const where = () =>
    page.evaluate(() => ({
      focused: document.activeElement.id || document.activeElement.tagName,
      scrollY: Math.round(scrollY),
    }));
  const topOf = (id) =>
    page.evaluate((id) => Math.round(document.getElementById(id).getBoundingClientRect().top), id);

  await page.goto(`${address}many-scenarios.html`);

  // Focus inside a block, in the window: the heading before it.
  await page.evaluate((id) => {
    const target = document.getElementById(id).parentElement.querySelector("button, a, [tabindex]") ??
      document.getElementById(id);
    target.scrollIntoView();
    target.focus();
  }, heading(4));
  await rebuild();
  assert.equal((await where()).focused, heading(4), "focus case");
  assert.equal(await topOf(heading(4)), 0, "focus case, scrolled to the heading");

  // Scrolled on with the mouse, the focused heading out of the window: the scroll counts.
  await page.evaluate((id) => {
    document.getElementById(id).scrollIntoView();
    scrollBy(0, 100);
  }, heading(7));
  await rebuild();
  assert.equal((await where()).focused, heading(7), "scroll case");
  assert.equal(await topOf(heading(7)), 0, "scroll case, scrolled to the heading");

  // At the top, with nothing focused: the page opens as any page does.
  await page.evaluate(() => {
    document.activeElement.blur();
    scrollTo(0, 0);
  });
  await rebuild();
  assert.equal((await where()).scrollY, 0, "top case");

  console.log("The reload keeps the reader's heading, in every case.");
} finally {
  await browser.close();
  server.kill();
  rmSync(folder, { recursive: true, force: true });
}
