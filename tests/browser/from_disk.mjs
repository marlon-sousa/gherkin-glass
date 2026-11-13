// docs/BEHAVIOUR.md, "A page opened from disk", rule 182, checked in Chromium
// with Playwright: tests/fixtures/pages is built, and pages of it opened
// straight from disk must show the notice first in their body. That a served
// page hides it, tests/browser/accessibility.mjs checks on every page.
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
import { chromium, ghg } from "./support.mjs";

const output = mkdtempSync(join(tmpdir(), "ghg-from-disk-"));
const browser = await chromium.launch();
try {
  const run = spawnSync(ghg, ["build", "--output", output], {
    cwd: resolve("tests/fixtures/pages"),
    encoding: "utf8",
  });
  assert.equal(run.status, 0, `ghg build in tests/fixtures/pages: ${run.stderr}`);
  const page = await browser.newPage();
  // The home page, and a page in a folder, whose script is further up.
  for (const path of ["index.html", "billing/refunds.html"]) {
    await page.goto(pathToFileURL(join(output, path)).href);
    const notice = await page.evaluate(() => {
      const first = document.body.firstElementChild;
      return { id: first.id, shown: !first.hidden && first.offsetHeight > 0, text: first.textContent };
    });
    assert.equal(notice.id, "ghg_from_disk", `${path}: the body does not start with the notice`);
    assert.ok(notice.shown, `${path}: the notice is not shown from disk`);
    assert.match(notice.text, /^This site is meant to be served, /, path);
  }
} finally {
  await browser.close();
  rmSync(output, { recursive: true, force: true });
}
console.log("Pages opened from disk show the notice.");
