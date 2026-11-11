// docs/BEHAVIOUR.md, "Accessibility", checked in Chromium with Playwright and
// axe-core: every fixture that builds is built into a site, served with
// ghg serve --no-build, and each page is checked with axe-core in the light
// and the dark colour scheme and in each theme a reader can pick, then driven
// with the keyboard.
//
// Run from the repository's root, after cargo build and npm ci in script/,
// with Playwright's Chromium installed:
//   node tests/browser/accessibility.mjs
// cargo xtask check-browser does all of it. GHG names another ghg binary than
// target/debug/ghg.

import { spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative, resolve, sep } from "node:path";
import assert from "node:assert/strict";
import { axeSource, chromium, ghg, serve, stop } from "./support.mjs";

const fixtures = resolve("tests/fixtures");

// Every build of a fixture that makes a site: the folder it runs in, below
// tests/fixtures, and its arguments after `ghg build`, to which the check adds
// --output.
const builds = [
  { folder: "anchors", args: ["--input", "."] },
  { folder: "constructs", args: ["--input", "."] },
  { folder: "doc-strings", args: ["--input", "."] },
  { folder: "input-files", args: ["--input", "."] },
  { folder: "pages", args: ["--config", "ghg.toml"] },
  { folder: "pages", args: ["--config", "collapsed.toml"] },
  { folder: "backgrounds", args: ["--config", "ghg.toml"] },
  { folder: "backgrounds", args: ["--config", "linked.toml"] },
  { folder: "tags", args: ["--config", "ghg.toml"] },
  { folder: "highlighting", args: ["--config", "ghg.toml"] },
  { folder: "configuration/default", args: [] },
  { folder: "configuration/configured", args: [] },
  { folder: "configuration/accents", args: [] },
  { folder: "configuration/configured/nested", args: [] },
  { folder: "configuration/unnamed-allowed", args: [] },
  { folder: "configuration/unnamed-allowed", args: ["--config", "allowed.toml"] },
  { folder: "serve/built", args: [] },
  { folder: "serve/reload", args: [] },
];

// The fixtures that make no site: those whose build is meant to fail, and
// those that are sites already rather than input.
const noSite = [
  "unreadable",
  "same-page",
  "tags-folder",
  "configuration/configured/ci",
  "configuration/empty-input",
  "configuration/faint-accents",
  "configuration/mistaken",
  "configuration/mistaken-pages",
  "configuration/mistaken-tags",
  "configuration/not-boolean",
  "configuration/not-colour",
  "configuration/not-toml",
  "configuration/unnamed",
  "pages/expected",
  "pages/expected-collapsed",
  "backgrounds/expected",
  "backgrounds/expected-linked",
  "tags/expected",
  "serve/site",
  "accessibility/failing",
];

// The rules the check fails on: WCAG 2.2 A and AA, and the enhanced contrast
// of AAA, leaving out the rules axe-core tags experimental or deprecated.
const tags = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];
const extraRules = ["color-contrast-enhanced"];

// The browser's colour scheme, light and dark, and each theme a reader can
// pick on the page, Light and Dark, each picked once, in the other scheme.
const passes = [
  { name: "light", scheme: "light" },
  { name: "dark", scheme: "dark" },
  { name: "the Dark theme", scheme: "light", theme: "dark" },
  { name: "the Light theme", scheme: "dark", theme: "light" },
];

// The files of `folder`, below it, with `/` between folders.
function filesOf(folder) {
  return readdirSync(folder, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => relative(folder, join(entry.parentPath, entry.name)).split(sep).join("/"));
}

const within = (path, folder) => path === folder || path.startsWith(`${folder}/`);

// Every feature file and configuration file under tests/fixtures belongs to a
// build or to a fixture that makes no site, so that a new fixture is not
// left out of the check unnoticed.
function checkEveryFixtureIsCounted() {
  const counted = [...builds.map((build) => build.folder), ...noSite];
  const missed = filesOf(fixtures)
    .filter((path) => /\.(feature|story|toml)$/.test(path))
    .filter((path) => !counted.some((folder) => within(path, folder)));
  assert.deepEqual(
    missed,
    [],
    "files of fixtures neither built by tests/browser/accessibility.mjs nor listed in its noSite",
  );
}

function build({ folder, args }, output) {
  const run = spawnSync(ghg, ["build", ...args, "--output", output], {
    cwd: join(fixtures, folder),
    encoding: "utf8",
  });
  assert.equal(run.status, 0, `ghg build ${args.join(" ")} in ${folder}: ${run.stderr}`);
}

// The address of a page below the site's address, its path's parts percent-encoded.
const addressOf = (site, page) => new URL(page.split("/").map(encodeURIComponent).join("/"), site).href;

// Runs axe-core on the page and returns its violations, each as one line per element.
async function violations(page) {
  await page.addScriptTag({ path: axeSource });
  const found = await page.evaluate(
    async ({ tags, extraRules }) => {
      const rules = window.axe
        .getRules(tags)
        .filter((rule) => !rule.tags.includes("experimental") && !rule.tags.includes("deprecated"))
        .map((rule) => rule.ruleId)
        .concat(extraRules);
      const results = await window.axe.run(document, {
        runOnly: { type: "rule", values: rules },
        resultTypes: ["violations"],
      });
      return results.violations.flatMap((violation) =>
        violation.nodes.map((node) => `${violation.id}: ${node.target.join(" ")}: ${node.failureSummary}`),
      );
    },
    { tags, extraRules },
  );
  return found;
}

// What has focus, as a short description that tells the page's elements apart.
const describeFocus = (page) =>
  page.evaluate(() => {
    const element = document.activeElement;
    if (!element || element === document.body) return "nothing";
    return element.dataset.ghgCheck ?? "an element the check did not list as focusable";
  });

// Tab from the start of the page reaches every link, button and other control
// the page shows, each once, in the order of the page, then leaves the page.
async function keyboardProblems(page) {
  const expected = await page.evaluate(() => {
    const focusable = [
      ...document.querySelectorAll(
        "a[href], button, input, select, textarea, summary, [tabindex], [contenteditable]",
      ),
    ].filter(
      (element) =>
        !element.disabled && element.tabIndex >= 0 && element.checkVisibility({ visibilityProperty: true }),
    ).filter((element) => {
      // A group of radio buttons is one Tab stop: its chosen button, or its first when none is.
      if (!(element instanceof HTMLInputElement) || element.type !== "radio" || !element.name) return true;
      const group = [...document.querySelectorAll(`input[type="radio"][name="${CSS.escape(element.name)}"]`)];
      return element.checked || (!group.some((radio) => radio.checked) && group[0] === element);
    });
    return focusable.map((element, index) => {
      const name = `${element.tagName.toLowerCase()} ${index + 1}, "${element.textContent.trim().slice(0, 60)}"`;
      element.dataset.ghgCheck = name;
      return name;
    });
  });
  const reached = [];
  for (let presses = 0; presses <= expected.length; presses += 1) {
    await page.keyboard.press("Tab");
    const focused = await describeFocus(page);
    if (focused === "nothing") break;
    reached.push(focused);
  }
  if (JSON.stringify(reached) === JSON.stringify(expected)) return [];
  return [
    `keyboard: Tab reached ${JSON.stringify(reached)}, where the page shows ${JSON.stringify(expected)}`,
  ];
}

// Enter and Space on the first button that shows or hides content, a block's
// or a folder page's feature's, each turn it over, and on a page with blocks
// Enter on "Collapse all" and "Expand all" acts as a click does. Returns null
// for a page without such a button, where there is nothing to press.
async function buttonProblems(page) {
  const problems = [];
  const button = page.locator(".ghg-toggle").first();
  if ((await button.count()) === 0) return null;
  const state = () => button.getAttribute("aria-expanded");
  const before = await state();
  await button.focus();
  await page.keyboard.press("Enter");
  const afterEnter = await state();
  await page.keyboard.press("Space");
  const afterSpace = await state();
  if (afterEnter === before || afterSpace !== before) {
    problems.push(
      `keyboard: the first block's button went from ${before} to ${afterEnter} with Enter, then ${afterSpace} with Space`,
    );
  }
  const blocks = page.locator(".ghg-block .ghg-toggle");
  if ((await blocks.count()) === 0) return problems;
  for (const [name, expanded] of [
    ["Collapse all", "false"],
    ["Expand all", "true"],
  ]) {
    const control = page.getByRole("button", { name, exact: true });
    if ((await control.count()) === 0) {
      problems.push(`keyboard: the page has blocks and no "${name}" button`);
      continue;
    }
    await control.focus();
    await page.keyboard.press("Enter");
    const states = await blocks.evaluateAll((buttons) =>
      buttons.map((element) => element.getAttribute("aria-expanded")),
    );
    if (states.some((value) => value !== expanded)) {
      problems.push(`keyboard: Enter on "${name}" left the blocks at ${JSON.stringify(states)}`);
    }
  }
  return problems;
}

// The folder tree is built once its data is fetched: the check waits for it,
// or for the checkbox to be disabled, before it counts what Tab reaches. A
// page without the tree's landmark, such as accessibility/failing's, has
// nothing to wait for.
const treeBuilt = (page) =>
  page.waitForFunction(() => {
    const landmark = document.querySelector("nav.ghg-tree");
    return (
      !landmark ||
      landmark.querySelector('[role="tree"]') !== null ||
      document.querySelector("input.ghg-show-tree")?.disabled === true
    );
  });

// Down from the tree's Tab stop moves the focus to the next item, and Up back,
// as rule 91 describes. Returns null for a page whose tree is not shown.
async function treeProblems(page) {
  const stop = page.locator('nav.ghg-tree [role="treeitem"][tabindex="0"]');
  if ((await stop.count()) === 0 || !(await stop.isVisible())) return null;
  const focusedText = () => page.evaluate(() => document.activeElement?.textContent ?? "");
  await stop.focus();
  const start = await focusedText();
  await page.keyboard.press("ArrowDown");
  const down = await focusedText();
  await page.keyboard.press("ArrowUp");
  const up = await focusedText();
  const last = (await page.locator('nav.ghg-tree [role="treeitem"]:visible').count()) === 1;
  if ((!last && down === start) || up !== start) {
    return [`keyboard: the folder tree went from ${JSON.stringify(start)} to ${JSON.stringify(down)} with Down, then ${JSON.stringify(up)} with Up`];
  }
  return [];
}

// Checks every page of the site in `folder`, served with ghg serve
// --no-build, and returns the problems, one line each, with the page's path.
export async function checkSite(browser, folder) {
  const pages = filesOf(folder)
    .filter((path) => path.endsWith(".html"))
    .sort();
  const { server, address } = await serve(["--no-build", "--output", folder, "--port", "0"], folder);
  const problems = [];
  let pressed = 0;
  let trees = 0;
  try {
    for (const { name, scheme, theme } of passes) {
      const context = await browser.newContext({ colorScheme: scheme });
      if (theme !== undefined) {
        // As if the reader had picked the theme on an earlier page.
        await context.addInitScript((picked) => {
          try {
            localStorage.setItem("gherkin-glass:theme", picked);
          } catch {
            // A document without storage, such as the first blank page.
          }
        }, theme);
      }
      const page = await context.newPage();
      for (const path of pages) {
        await page.goto(addressOf(address, path));
        await page.emulateMedia({ colorScheme: scheme });
        await treeBuilt(page);
        if (theme !== undefined) {
          const shown = await page.evaluate(() => document.documentElement.dataset.theme);
          if (shown !== theme) problems.push(`${path}, ${name}: the page shows the theme ${shown}`);
        }
        for (const line of await violations(page)) problems.push(`${path}, ${name}: ${line}`);
        if (name === "light") {
          for (const line of await keyboardProblems(page)) problems.push(`${path}: ${line}`);
          const found = await buttonProblems(page);
          if (found !== null) pressed += 1;
          for (const line of found ?? []) problems.push(`${path}: ${line}`);
          const tree = await treeProblems(page);
          if (tree !== null) trees += 1;
          for (const line of tree ?? []) problems.push(`${path}: ${line}`);
          // What the buttons remembered is forgotten, so that it never
          // changes how another page starts.
          await page.evaluate(() => localStorage.clear());
        }
      }
      await context.close();
    }
  } finally {
    await stop(server);
  }
  return { pages: pages.length, pressed, trees, problems };
}

async function main() {
  checkEveryFixtureIsCounted();
  const browser = await chromium.launch();
  const output = mkdtempSync(join(tmpdir(), "ghg-accessibility-"));
  const failures = [];
  let checked = 0;
  let pressed = 0;
  let trees = 0;
  try {
    for (const [index, each] of builds.entries()) {
      const site = join(output, String(index));
      build(each, site);
      const name = [each.folder, ...each.args].join(" ");
      const result = await checkSite(browser, site);
      if (result.pages === 0) failures.push(`${name}: the site has no page`);
      checked += result.pages;
      pressed += result.pressed;
      trees += result.trees;
      const { problems } = result;
      for (const problem of problems) failures.push(`${name}: ${problem}`);
    }
    if (pressed === 0) failures.push("no page had a block whose buttons the check could press");
    if (trees === 0) failures.push("no page showed a folder tree the check could move through");
    // The check must fail on a page that breaks each of its rules.
    const { problems } = await checkSite(browser, join(fixtures, "accessibility/failing"));
    const found = (start) => problems.some((line) => line.startsWith(`index.html${start}`));
    for (const [start, what] of [
      [", light: color-contrast-enhanced: .aa-only:", "enhanced contrast, in the light scheme"],
      [", dark: color-contrast: .dark-only:", "contrast, in the dark scheme"],
      [", light: image-alt:", "a WCAG 2 A rule"],
      [": keyboard: Tab reached", "the keyboard order"],
      [": keyboard: the first block's button", "a block's button that Space does not turn over"],
    ]) {
      if (!found(start)) failures.push(`tests/fixtures/accessibility/failing: the check did not find ${what}`);
    }
    if (found(", light: color-contrast: .dark-only:")) {
      failures.push("tests/fixtures/accessibility/failing: the dark scheme's contrast failed in the light scheme");
    }
  } finally {
    await browser.close();
    rmSync(output, { recursive: true, force: true });
  }
  if (failures.length > 0) {
    console.error(failures.join("\n"));
    process.exit(1);
  }
  console.log(`Every page passes the accessibility check: ${checked} pages, in ${builds.length} sites.`);
}

await main();
