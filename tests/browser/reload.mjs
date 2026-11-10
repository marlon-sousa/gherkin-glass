// docs/BEHAVIOUR.md, "Reloading", rules 71 and 72, checked in Chromium with
// Playwright: ghg serve --watch serves a copy of tests/fixtures/serve/reload,
// a feature changes, and the page must reload at the reader's heading.
//
// Run from the repository's root, after cargo build and npm ci in script/,
// with Playwright's Chromium installed:
//   node tests/browser/reload.mjs
// cargo xtask check-browser does all of it. GHG names another ghg binary than
// target/debug/ghg.

import { spawn } from "node:child_process";
import { cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createInterface } from "node:readline";
import assert from "node:assert/strict";
import { chromium, ghg } from "./support.mjs";

const folder = mkdtempSync(join(tmpdir(), "ghg-reload-"));
cpSync("tests/fixtures/serve/reload", folder, { recursive: true });
const feature = join(folder, "features/many-scenarios.feature");

// ghg serve --watch at `port`, and the address it prints.
async function serve(port) {
  const server = spawn(ghg, ["serve", "--watch", "--port", String(port)], { cwd: folder });
  const firstLine = await new Promise((done) =>
    createInterface({ input: server.stdout }).once("line", done),
  );
  const address = /^The site is served at (\S+)\. /.exec(firstLine)?.[1];
  assert.ok(address, firstLine);
  return { server, address };
}

let { server, address } = await serve(0);

const browser = await chromium.launch();
let build = 1;

try {
  const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
  const heading = (n) => `scenario-number-${n}`;

  // Waits for the page of version `build`.
  async function reloaded() {
    await page.waitForFunction(
      (number) =>
        document.querySelector('script[src*="/__ghg/reload.js"]')?.src.endsWith(`build=${number}`),
      build,
      { timeout: 15000 },
    );
    await page.waitForLoadState("load");
    await page.waitForTimeout(200);
  }

  // Changes a feature, then waits for the page of the next version.
  async function rebuild(change = (text) => `${text}\n`) {
    build += 1;
    writeFileSync(feature, change(readFileSync(feature, "utf8")));
    await reloaded();
  }

  const where = () =>
    page.evaluate(() => ({
      focused: document.activeElement.id || document.activeElement.tagName,
      scrollY: Math.round(scrollY),
    }));
  const topOf = (id) =>
    page.evaluate((id) => Math.round(document.getElementById(id).getBoundingClientRect().top), id);

  await page.goto(`${address}many-scenarios.html`);

  // Focus in the window wins over the scroll: heading 4's button is focused
  // in the middle of the window, while heading 3's top is above it.
  await page.evaluate((id) => {
    const button = document.querySelector(`#${id} button`);
    button.focus({ preventScroll: true });
    document.getElementById(id).scrollIntoView({ block: "center" });
  }, heading(4));
  assert.ok((await topOf(heading(3))) < 0, "focus case, heading 3 above the window");
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

  assert.equal((await where()).focused, "BODY", "top case, nothing focused");

  // A heading the page no longer has: the page opens as any page does.
  await page.evaluate((id) => document.getElementById(id).scrollIntoView(), heading(5));
  await rebuild((text) => text.replace("Scenario number 5", "Scenario renamed"));
  assert.equal((await where()).focused, "BODY", "renamed case, nothing focused");

  // While ghg serve does not answer, the script keeps asking.
  const port = new URL(address).port;
  server.kill();
  await new Promise((done) => server.once("exit", done));
  await page.waitForTimeout(2500);
  ({ server, address } = await serve(port));
  build = 1;
  await reloaded();

  console.log("The reload keeps the reader's heading, in every case.");
} finally {
  await browser.close();
  server.kill();
  rmSync(folder, { recursive: true, force: true });
}
