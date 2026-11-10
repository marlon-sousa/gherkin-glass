// The policy of src/policies/theme_choice.ts, and the page's own reading of the kept choice:
// docs/BEHAVIOUR.md, rules 61 to 63.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { THEME_KEY, keptChoice, themeAttribute } from "../../src/policies/theme_choice.ts";

test("light and dark are kept, and anything else is the system's choice", () => {
  assert.equal(keptChoice("light"), "light");
  assert.equal(keptChoice("dark"), "dark");
  assert.equal(keptChoice("system"), "system");
  assert.equal(keptChoice(null), "system");
  assert.equal(keptChoice("Dark"), "system");
});

test("the system's choice sets no theme on the page", () => {
  assert.equal(themeAttribute("system"), null);
  assert.equal(themeAttribute("light"), "light");
  assert.equal(themeAttribute("dark"), "dark");
});

test("the page's head reads the choice under the same key, before it is drawn", () => {
  const layout = readFileSync(new URL("../../../templates/layout.html", import.meta.url), "utf8");
  const head = layout.slice(0, layout.indexOf("<link rel=\"stylesheet\""));
  assert.ok(head.includes(`localStorage.getItem("${THEME_KEY}")`), head);
  assert.ok(!head.includes("type=\"module\""), "the head's script is not deferred");
});

test("no block's state can be kept under the theme's key", () => {
  assert.ok(!THEME_KEY.startsWith("gherkin-glass:/"));
});
