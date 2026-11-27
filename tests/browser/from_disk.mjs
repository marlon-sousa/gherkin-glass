// docs/BEHAVIOUR.md, "A page opened from disk", rule 182, and rule 242,
// checked in Chromium with Playwright: tests/fixtures/pages is built, and so
// are the versions of tests/fixtures/versions, and their shells and the
// versions page, opened straight from disk, must open the dialog first in
// their body, with the focus on the dialog itself. I agree closes it on one
// page and Escape on the next, and Tab then reaches the skip link. A shell
// whose address names a page other than its front page, and a version's
// shell, say in their main that the page cannot be shown. The open dialog is
// held to the rules of "Accessibility", in the light and the dark colour
// scheme. That a served page keeps it closed, tests/browser/accessibility.mjs
// checks on every page. A shell opened with JavaScript turned off shows the
// paragraph of its noscript element.
//
// Run from the repository's root, after cargo build and npm ci in script/,
// with Playwright's Chromium installed:
//   node tests/browser/from_disk.mjs
// cargo xtask check-browser does all of it. GHG names another ghg binary than
// target/debug/ghg.

import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import assert from "node:assert/strict";
import { partProblems } from "./part_contrast.mjs";
import { axeSource, chromium, ghg, violations } from "./support.mjs";

const CANNOT = "This page cannot be shown when opened from disk. Serve the site, then open the address the server gives.";
const NOSCRIPT = "This site needs JavaScript to show its pages. Turn it on, then load this page again.";

// Runs ghg with `args` in tests/fixtures/`folder`, into `output`.
const build = (folder, args, output) => {
  const run = spawnSync(ghg, [...args, "--output", output], { cwd: resolve("tests/fixtures", folder), encoding: "utf8" });
  assert.equal(run.status, 0, `ghg ${args.join(" ")} in tests/fixtures/${folder}: ${run.stderr}`);
};

const output = mkdtempSync(join(tmpdir(), "ghg-from-disk-"));
const site = join(output, "pages");
const versioned = join(output, "versions");
// Each page opened, in the site it is in, and whether its main says it cannot be shown.
const opened = [
  { folder: site, path: "index.html", hash: "", cannot: false },
  { folder: site, path: "index.html", hash: "#billing/refunds", cannot: true },
  { folder: versioned, path: "index.html", hash: "", cannot: false },
  { folder: versioned, path: "v1.0.0/index.html", hash: "#", cannot: true },
];
const browser = await chromium.launch();
try {
  build("pages", ["build"], site);
  build("versions/suite", ["version", "add", "v1.0.0"], versioned);
  const problems = [];
  for (const scheme of ["light", "dark"]) {
    const context = await browser.newContext({ colorScheme: scheme });
    await context.addInitScript({ path: axeSource });
    const page = await context.newPage();
    const dialog = () =>
      page.evaluate(() => {
        const first = document.body.firstElementChild;
        return {
          id: first.id,
          open: first.open,
          modal: first.matches(":modal"),
          text: first.textContent,
          focused: document.activeElement === first,
        };
      });
    for (const [place, { folder, path, hash, cannot }] of opened.entries()) {
      const name = `${folder === site ? "pages" : "versions"}/${path}${hash}`;
      await page.goto("about:blank");
      await page.goto(`${pathToFileURL(join(folder, path)).href}${hash}`);
      const shown = await dialog();
      assert.equal(shown.id, "ghg_from_disk", `${name}: the body does not start with the dialog`);
      assert.ok(shown.open && shown.modal, `${name}: the dialog is not open as a modal dialog`);
      assert.match(shown.text, /This site is meant to be served, /, name);
      assert.ok(shown.focused, `${name}: the focus is not on the dialog`);
      const said = await page.evaluate(() => document.querySelector("main")?.textContent.trim());
      if (cannot) assert.equal(said, CANNOT, `${name}: the main does not say the page cannot be shown`);
      else assert.notEqual(said, CANNOT, `${name}: the main says the page cannot be shown`);
      // Tab moves from the dialog to its button, the one control inside it.
      await page.keyboard.press("Tab");
      assert.ok(
        await page.evaluate(() => document.activeElement.textContent === "I agree"),
        `${name}: Tab from the dialog does not reach I agree`,
      );
      for (const line of await violations(page)) problems.push(`${name}, ${scheme}: ${line}`);
      // Last before closing, since it moves the focus.
      for (const line of (await partProblems(page)).lines) problems.push(`${name}, ${scheme}: ${line}`);
      const close = place % 2 === 0 ? "I agree" : "Escape";
      if (close === "I agree") {
        await page.focus("#ghg_from_disk button");
        await page.keyboard.press("Enter");
      } else {
        await page.keyboard.press("Escape");
      }
      assert.equal((await dialog()).open, false, `${name}: ${close} does not close the dialog`);
      await page.keyboard.press("Tab");
      assert.ok(
        await page.evaluate(() => document.activeElement.classList.contains("ghg-skip")),
        `${name}: after ${close}, Tab does not reach the skip link`,
      );
    }
    await context.close();
  }
  assert.deepEqual(problems, [], "the open dialog breaks the rules of Accessibility");
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  await page.goto(pathToFileURL(join(site, "index.html")).href);
  const noscript = await page.locator("noscript p").textContent();
  assert.equal(noscript, NOSCRIPT, "the shell's noscript element");
  assert.ok(await page.locator("noscript p").isVisible(), "the paragraph of the noscript element is not shown");
  await context.close();
} finally {
  await browser.close();
  rmSync(output, { recursive: true, force: true });
}
console.log("Pages opened from disk open the dialog, and I agree and Escape close it; a shell without JavaScript says why it shows nothing.");
