// docs/BEHAVIOUR.md, "What a page holds", rule 203, checked in Chromium with
// Playwright: tests/fixtures/hostile, whose feature files hold markup and
// scripts in every place their text reaches, is built and served, and on its
// feature page the folder tree is opened all the way down and the hostile
// words are searched for. What the script writes, the tree's items and the
// results, must read the text as written in the feature files, hold only the
// elements the script itself makes, and no dialog may open, as one would if
// any of the feature files' scripts ran.
//
// Run from the repository's root, after cargo build and npm ci in script/,
// with Playwright's Chromium installed:
//   node tests/browser/hostile.mjs
// cargo xtask check-browser does all of it. GHG names another ghg binary than
// target/debug/ghg.

import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import assert from "node:assert/strict";
import { chromium, ghg, serve, stop } from "./support.mjs";

// The elements the script makes for the folder tree and the search results.
const scriptElements = ["a", "bdi", "li", "ol", "span", "ul"];

// What the tree and the results must read, as written in the feature files.
const treeTexts = [
  "Q&A's notes",
  "Feature: Answers & <b>questions</b>",
  "Scenario: <marquee>Moving</marquee> & 'still'",
  "Feature: Hostile <script>alert(\"feature\")</script> & 'friends'",
  "Scenario: Scenario <img src=x onerror=alert('scenario')> & \"quoted\"",
  "Rule: Rule <svg onload=\"alert('rule')\"></svg>",
  "Scenario outline: Outline <object data=\"x\"></object> for <who>",
];
const searches = [
  { words: "Hostile", reads: "Hostile <script>alert(\"feature\")</script> & 'friends', feature" },
  { words: "still", reads: "<marquee>Moving</marquee> & 'still', scenario in Answers & <b>questions</b>" },
  {
    words: "onload",
    reads: "Rule <svg onload=\"alert('rule')\"></svg>, rule in Hostile <script>alert(\"feature\")</script> & 'friends'",
  },
];

const output = mkdtempSync(join(tmpdir(), "ghg-hostile-"));
const browser = await chromium.launch();
let server = null;
try {
  const run = spawnSync(ghg, ["build", "--output", output], {
    cwd: resolve("tests/fixtures/hostile"),
    encoding: "utf8",
  });
  assert.equal(run.status, 0, `ghg build in tests/fixtures/hostile: ${run.stderr}`);
  const served = await serve(["--no-build", "--output", output, "--port", "0"], output);
  server = served.server;
  const page = await browser.newPage();
  const dialogs = [];
  page.on("dialog", async (dialog) => {
    dialogs.push(dialog.message());
    await dialog.dismiss();
  });
  // The elements below `selector`, by tag name, the script's own left out.
  const foreign = (selector) =>
    page.evaluate(
      ({ selector, allowed }) =>
        [...document.querySelectorAll(`${selector} *`)]
          .map((element) => element.localName)
          .filter((name) => !allowed.includes(name)),
      { selector, allowed: scriptElements },
    );
  // The text of each element `selector` finds.
  const texts = (selector) =>
    page.locator(selector).evaluateAll((elements) => elements.map((element) => element.textContent));

  await page.goto(new URL("hostile.html", served.address).href);
  await page.waitForFunction(() => document.querySelector('nav.ghg-tree [role="tree"]') !== null);
  const tree = await texts('nav.ghg-tree a[role="treeitem"]');
  for (const text of treeTexts) {
    assert.ok(
      tree.some((item) => item.includes(text)),
      `the folder tree does not read ${JSON.stringify(text)}: ${JSON.stringify(tree)}`,
    );
  }
  assert.deepEqual(await foreign("nav.ghg-tree"), [], "the folder tree holds elements the script does not make");

  for (const { words, reads } of searches) {
    await page.locator(".ghg-search-field").fill(words);
    await page.keyboard.press("Enter");
    await page.waitForFunction(() => document.querySelectorAll(".ghg-search-results a").length > 0);
    const results = await texts(".ghg-search-results a");
    assert.ok(
      results.includes(reads),
      `searching ${words} does not read ${JSON.stringify(reads)}: ${JSON.stringify(results)}`,
    );
    assert.deepEqual(
      await foreign(".ghg-search-results"),
      [],
      `the results of ${words} hold elements the script does not make`,
    );
    await page.keyboard.press("Escape");
  }
  assert.deepEqual(dialogs, [], "a script of the feature files ran and opened a dialog");
} finally {
  if (server !== null) await stop(server);
  await browser.close();
  rmSync(output, { recursive: true, force: true });
}
console.log("Markup in feature files reaches the folder tree and the search results as text, and runs nothing.");
