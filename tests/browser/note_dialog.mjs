// docs/BEHAVIOUR.md, "The note of a keyword", rules 196 to 198, checked in
// Chromium with Playwright: tests/fixtures/pages is built and served, and on
// kinds.html, whose blocks are written with other keywords, the button of a
// note, reached with the keyboard, must open the page's dialog as a modal
// dialog with the focus on the dialog itself; Tab then reaches Got it. The
// open dialog is held to the rules of "Accessibility", in the light and the
// dark colour scheme. Got it closes it in one scheme and Escape in the
// other, and each gives the focus back to the note's button.
//
// Run from the repository's root, after cargo build and npm ci in script/,
// with Playwright's Chromium installed:
//   node tests/browser/note_dialog.mjs
// cargo xtask check-browser does all of it. GHG names another ghg binary than
// target/debug/ghg.

import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import assert from "node:assert/strict";
import { partProblems } from "./part_contrast.mjs";
import { axeSource, chromium, ghg, serve, stop, violations } from "./support.mjs";

const output = mkdtempSync(join(tmpdir(), "ghg-note-dialog-"));
const browser = await chromium.launch();
let server = null;
try {
  const run = spawnSync(ghg, ["build", "--output", output], {
    cwd: resolve("tests/fixtures/pages"),
    encoding: "utf8",
  });
  assert.equal(run.status, 0, `ghg build in tests/fixtures/pages: ${run.stderr}`);
  const served = await serve(["--no-build", "--output", output, "--port", "0"], output);
  server = served.server;
  const problems = [];
  for (const [scheme, close] of [
    ["light", "Got it"],
    ["dark", "Escape"],
  ]) {
    const context = await browser.newContext({ colorScheme: scheme });
    await context.addInitScript({ path: axeSource });
    const page = await context.newPage();
    await page.goto(new URL("kinds.html", served.address).href);
    const dialog = () =>
      page.evaluate(() => {
        const dialog = document.getElementById("ghg_note_dialog");
        return {
          open: dialog.open,
          modal: dialog.matches(":modal"),
          name: dialog.getAttribute("aria-labelledby"),
          focused: document.activeElement === dialog,
        };
      });
    assert.equal((await dialog()).open, false, `${scheme}: the dialog is open as the page loads`);
    // The first note's button, the feature's, under the h1.
    const opener = page.locator("button.ghg-note-opener").first();
    await opener.focus();
    await page.keyboard.press("Enter");
    const opened = await dialog();
    assert.ok(opened.open && opened.modal, `${scheme}: the note's button does not open the dialog as a modal dialog`);
    assert.equal(opened.name, "ghg_note_dialog_title");
    assert.ok(opened.focused, `${scheme}: the focus is not on the dialog`);
    await page.keyboard.press("Tab");
    assert.ok(
      await page.evaluate(() => document.activeElement.textContent === "Got it"),
      `${scheme}: Tab from the dialog does not reach Got it`,
    );
    for (const line of await violations(page)) problems.push(`kinds.html, ${scheme}: ${line}`);
    for (const line of (await partProblems(page)).lines) problems.push(`kinds.html, ${scheme}: ${line}`);
    if (close === "Got it") {
      await page.focus("#ghg_note_dialog button");
      await page.keyboard.press("Enter");
    } else {
      await page.keyboard.press("Escape");
    }
    assert.equal((await dialog()).open, false, `${scheme}: ${close} does not close the dialog`);
    assert.ok(
      await opener.evaluate((button) => document.activeElement === button),
      `${scheme}: after ${close}, the focus is not back on the note's button`,
    );
    await context.close();
  }
  assert.deepEqual(problems, [], "the open dialog breaks the rules of Accessibility");
} finally {
  if (server !== null) await stop(server);
  await browser.close();
  rmSync(output, { recursive: true, force: true });
}
console.log("A note's button opens its dialog, and Got it and Escape close it, giving the focus back.");
