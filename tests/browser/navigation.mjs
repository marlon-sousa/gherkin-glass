// docs/BEHAVIOUR.md, "How a page is drawn", rule 240, with rules 235 to 239,
// checked in Chromium with Playwright on the shells of tests/fixtures/pages,
// tests/fixtures/front-page and the history of versions of
// tests/fixtures/versions/history, built and served: the shell opened at an address draws its page,
// sets its title, says nothing and moves the focus only to the anchor's
// element; a link followed to another page, the browser's Back, a link to an
// anchor of another page and one of the page shown, and the skip link, each
// leave the focus, the title, the live region, the address and the scroll as
// the rule says. Every page drawn is held to the rules of "Accessibility" in
// the light and the dark colour scheme. A shell holding a front page puts it
// back, and the search box with it, after other pages; a version's shell
// opens its lists in other versions, held to the same rules.
//
// Run from the repository's root, after cargo build and npm ci in script/,
// with Playwright's Chromium installed:
//   node tests/browser/navigation.mjs
// cargo xtask check-browser does all of it. GHG names another ghg binary than
// target/debug/ghg.

import assert from "node:assert/strict";

import { axeSource, chromium, serveBuilt, violations } from "./support.mjs";

// Longer than the 150 milliseconds the script waits before it moves the focus.
const SETTLED = 400;

const problems = [];
const expect = (what, found, wanted) => {
  for (const [key, value] of Object.entries(wanted)) {
    if (JSON.stringify(found[key]) !== JSON.stringify(value)) {
      problems.push(`${what}: ${key} is ${JSON.stringify(found[key])}, not ${JSON.stringify(value)}`);
    }
  }
};

// What the reader meets once a page is drawn.
const seen = (page) =>
  page.evaluate(() => {
    const focused = document.activeElement;
    const box = focused?.getBoundingClientRect();
    return {
      title: document.title,
      heading: document.querySelector("main h1")?.textContent ?? null,
      said: document.getElementById("ghg_announcement")?.textContent ?? null,
      focus: focused === document.body ? "body" : focused?.id ? `#${focused.id}` : focused?.tagName.toLowerCase(),
      inView: box !== undefined && box.top > -1 && box.top < window.innerHeight,
      hash: location.hash,
      busy: document.querySelector("main")?.getAttribute("aria-busy") ?? null,
      lang: document.documentElement.lang,
    };
  });

const site = await serveBuilt([{ folder: "pages", args: ["build", "--config", "ghg.toml"] }]);
const frontPageSite = await serveBuilt([{ folder: "front-page", args: ["build", "--config", "ghg.toml"] }]);
const versionSite = await serveBuilt([
  { folder: "versions/history/first", args: ["version", "add", "v1.0.0"] },
  { folder: "versions/history/moved", args: ["version", "add", "v1.1.0"] },
  { folder: "versions/history/changed", args: ["version", "add", "v2.0.0"] },
]);
const browser = await chromium.launch();
try {
  for (const scheme of ["light", "dark"]) {
    const context = await browser.newContext({ colorScheme: scheme, viewport: { width: 1366, height: 850 } });
    await context.addInitScript({ path: axeSource });
    const page = await context.newPage();
    page.on("pageerror", (error) => problems.push(`${scheme}: the script threw: ${error.message}`));
    const check = async (what) => {
      for (const found of await violations(page)) problems.push(`${scheme}, ${what}: ${found}`);
    };

    // Opened at an address: the title is set, nothing is said, and the focus stays where the browser put it.
    await page.goto(`${site.address}index.html#billing/refunds`);
    await page.waitForTimeout(SETTLED);
    expect(`${scheme}, opened at a page`, await seen(page), {
      title: "Feature: Refunds & <returns>, billing/refunds.feature",
      heading: "Feature: Refunds & <returns>",
      said: "",
      focus: "body",
      busy: null,
      lang: "en",
    });
    await check("a feature's page");
    await page.keyboard.press("Tab");
    const first = await page.evaluate(() => document.activeElement?.textContent);
    if (first !== "Skip to content") problems.push(`${scheme}: the first Tab reaches ${JSON.stringify(first)}, not the skip link`);

    // Opened at an anchor inside a collapsed rule: the rule is expanded and the anchor's heading has the focus, in view.
    await page.goto(`${site.address}index.html#billing/refunds#store-credit`);
    await page.reload();
    await page.waitForTimeout(SETTLED);
    expect(`${scheme}, opened at an anchor`, await seen(page), { focus: "#store-credit", inView: true, said: "" });

    // A link of the breadcrumb, outside the main: the title is said and the focus is on the h1.
    await page.click('nav.ghg-breadcrumb a[href="#billing/"]');
    await page.waitForTimeout(SETTLED);
    expect(`${scheme}, the breadcrumb followed`, await seen(page), {
      title: "billing, Banking & <more>",
      heading: "billing",
      said: "billing, Banking & <more>",
      focus: "h1",
      inView: true,
      hash: "#billing/",
    });
    await check("a folder's page");

    // The browser's Back draws the page before.
    await page.goBack();
    await page.waitForTimeout(SETTLED);
    expect(`${scheme}, Back`, await seen(page), { heading: "Feature: Refunds & <returns>", said: "Feature: Refunds & <returns>, billing/refunds.feature", focus: "#store-credit" });

    // A link to an anchor of the page shown draws nothing again, and moves the focus at once.
    await page.evaluate(() => {
      document.querySelector("main h1").dataset.kept = "yes";
      location.hash = "#billing/refunds#refund-a-refund";
    });
    await page.waitForTimeout(50);
    const sameDocument = await page.evaluate(() => document.querySelector("main h1").dataset.kept);
    if (sameDocument !== "yes") problems.push(`${scheme}: a link to an anchor of the page shown drew the page again`);
    expect(`${scheme}, an anchor of the page shown`, await seen(page), { focus: "#refund-a-refund", inView: true });

    // The skip link names the main, moves the focus there, and leaves the address as it was.
    await page.locator("a.ghg-skip").focus();
    await page.keyboard.press("Enter");
    await page.waitForTimeout(50);
    expect(`${scheme}, the skip link`, await seen(page), { focus: "#ghg_main", hash: "#billing/refunds#refund-a-refund" });

    // A link of the folder tree to another page, then the other pages of the site, each held to the rules.
    await page.click('nav.ghg-tree a[href="#outlines"]');
    await page.waitForTimeout(SETTLED);
    expect(`${scheme}, the tree followed`, await seen(page), { heading: "Feature: Withdraw cash at the counter", focus: "h1", said: "Feature: Withdraw cash at the counter, outlines.feature" });
    const current = await page.evaluate(() => document.querySelector('nav.ghg-tree [aria-current="page"]')?.getAttribute("href"));
    if (current !== "#outlines") problems.push(`${scheme}: the tree marks ${JSON.stringify(current)} as the current page, not #outlines`);
    await check("an outline's page");
    for (const [hash, heading] of [
      ["#", "Banking & <more>"],
      ["#tags/", "Tags"],
      ["#tags/billing", "Tag @billing"],
      ["#steps", "Step sentences"],
      ["#nowhere", "Nothing is at this address"],
    ]) {
      await page.evaluate((to) => (location.hash = to), hash);
      await page.waitForTimeout(SETTLED);
      expect(`${scheme}, ${hash}`, await seen(page), { heading, focus: "h1" });
      await check(`the page at ${hash}`);
    }

    // The search box: Enter with no result chosen opens the page of search results, its words in the field.
    const field = page.locator("input.ghg-search-field");
    await field.fill("refund");
    await page.waitForTimeout(SETTLED);
    await field.press("Enter");
    await page.waitForTimeout(SETTLED);
    expect(`${scheme}, the page of search results`, await seen(page), { hash: "#search?q=refund", heading: 'Results for "refund"', title: 'Results for "refund", Banking & <more>', focus: "h1" });
    const inMain = await page.evaluate(() => document.querySelector("main form.ghg-search") !== null && document.querySelectorAll("form.ghg-search").length === 1);
    if (!inMain) problems.push(`${scheme}: the page of search results does not hold the one search box in its main`);
    await page.waitForFunction(() => document.querySelector(".ghg-search-groups h2") !== null);
    await check("the page of search results");
    await page.evaluate(() => (location.hash = "#billing/"));
    await page.waitForTimeout(SETTLED);
    const back = await page.evaluate(() => document.querySelector("form.ghg-search")?.nextElementSibling?.id);
    if (back !== "ghg_main") problems.push(`${scheme}: the search box is not put back before the main`);

    // A version's feature page: its lists in other versions open, and are held to the rules.
    await page.goto(`${versionSite.address}v2.0.0/index.html#refunds`);
    await page.waitForTimeout(SETTLED);
    const opener = page.locator("main h1 + .ghg-other-versions > button");
    await opener.click();
    const expanded = await opener.getAttribute("aria-expanded");
    if (expanded !== "true") problems.push(`${scheme}: the page's list in other versions is not expanded once its button is pressed`);
    await page.locator(".ghg-block .ghg-other-versions > button:visible").first().click();
    await check("a version's feature page, its lists open");
    await context.close();
  }

  // A shell holding a front page puts it back after other pages, as it arrived, the search box back in its place.
  const page = await browser.newPage();
  page.on("pageerror", (error) => problems.push(`the front page's shell: the script threw: ${error.message}`));
  await page.goto(`${frontPageSite.address}index.html#`);
  await page.waitForTimeout(SETTLED);
  expect("the front page, opened", await seen(page), { heading: "The Banking suite", title: "The Banking suite" });
  for (const hash of ["#search?q=refund", "#billing/", "#search?q=refund"]) {
    await page.evaluate((to) => (location.hash = to), hash);
    await page.waitForTimeout(SETTLED);
    await page.evaluate(() => (location.hash = "#"));
    await page.waitForTimeout(SETTLED);
    expect(`the front page, after ${hash}`, await seen(page), { heading: "The Banking suite", title: "The Banking suite", focus: "h1", said: "The Banking suite" });
    const shape = await page.evaluate(() => ({ headings: document.querySelectorAll("main h1").length, box: document.querySelector("form.ghg-search")?.nextElementSibling?.id ?? null }));
    expect(`the front page, after ${hash}`, shape, { headings: 1, box: "ghg_main" });
  }
} finally {
  await browser.close();
  await site.stop();
  await frontPageSite.stop();
  await versionSite.stop();
}
assert.deepEqual(problems, [], `navigation in a shell:\n${problems.join("\n")}`);
console.log("Navigation in a shell: the focus, the title, the live region and the scroll are as rule 240 says.");
