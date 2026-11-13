// docs/BEHAVIOUR.md, "A page opened from disk", rule 182, checked in Chromium
// with Playwright: tests/fixtures/pages is built, and pages of it opened
// straight from disk must open the dialog first in their body, with the focus
// on the dialog itself. I agree closes it on one page and Escape on the
// other, and Tab then reaches the skip link. The open dialog is held to the
// rules of "Accessibility", in the light and the dark colour scheme. That a
// served page keeps it closed, tests/browser/accessibility.mjs checks on
// every page.
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

const output = mkdtempSync(join(tmpdir(), "ghg-from-disk-"));
const browser = await chromium.launch();
try {
  const run = spawnSync(ghg, ["build", "--output", output], {
    cwd: resolve("tests/fixtures/pages"),
    encoding: "utf8",
  });
  assert.equal(run.status, 0, `ghg build in tests/fixtures/pages: ${run.stderr}`);
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
    // The home page, and a page in a folder, whose script is further up.
    for (const path of ["index.html", "billing/refunds.html"]) {
      await page.goto(pathToFileURL(join(output, path)).href);
      const opened = await dialog();
      assert.equal(opened.id, "ghg_from_disk", `${path}: the body does not start with the dialog`);
      assert.ok(opened.open && opened.modal, `${path}: the dialog is not open as a modal dialog`);
      assert.match(opened.text, /This site is meant to be served, /, path);
      assert.ok(opened.focused, `${path}: the focus is not on the dialog`);
      // Tab moves from the dialog to its button, the one control inside it.
      await page.keyboard.press("Tab");
      assert.ok(
        await page.evaluate(() => document.activeElement.textContent === "I agree"),
        `${path}: Tab from the dialog does not reach I agree`,
      );
      for (const line of await violations(page)) problems.push(`${path}, ${scheme}: ${line}`);
      // Last before closing, since it moves the focus.
      for (const line of (await partProblems(page)).lines) problems.push(`${path}, ${scheme}: ${line}`);
      // The home page's dialog closes with I agree, the other's with Escape.
      const close = path === "index.html" ? "I agree" : "Escape";
      if (close === "I agree") {
        await page.focus("#ghg_from_disk button");
        await page.keyboard.press("Enter");
      } else {
        await page.keyboard.press("Escape");
      }
      assert.equal((await dialog()).open, false, `${path}: ${close} does not close the dialog`);
      await page.keyboard.press("Tab");
      assert.ok(
        await page.evaluate(() => document.activeElement.classList.contains("ghg-skip")),
        `${path}: after ${close}, Tab does not reach the skip link`,
      );
    }
    await context.close();
  }
  assert.deepEqual(problems, [], "the open dialog breaks the rules of Accessibility");
} finally {
  await browser.close();
  rmSync(output, { recursive: true, force: true });
}
console.log("Pages opened from disk open the dialog, and I agree and Escape close it.");
