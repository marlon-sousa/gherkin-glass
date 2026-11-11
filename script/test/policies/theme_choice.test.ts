// The policy of src/policies/theme_choice.ts, and the page's own reading of the kept choice:
// docs/BEHAVIOUR.md, rules 114 to 116.

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

test("the page's head applies the kept choice as the policy does, before the page is drawn", () => {
  const layout = readFileSync(new URL("../../../templates/layout.html", import.meta.url), "utf8");
  const head = layout.slice(0, layout.indexOf("<style>"));
  const found = /<script>(.*)<\/script>/.exec(head);
  assert.ok(found?.[1], "a classic script before the page's style");
  const source = found[1];
  const run = (storage: { getItem(key: string): string | null }): string | undefined => {
    const root = { dataset: {} as { theme?: string } };
    new Function("localStorage", "document", source)(storage, { documentElement: root });
    return root.dataset.theme;
  };
  for (const kept of ["light", "dark", "system", "Dark", "", null]) {
    const storage = { getItem: (key: string) => (key === THEME_KEY ? kept : null) };
    assert.equal(run(storage), themeAttribute(keptChoice(kept)) ?? undefined, String(kept));
  }
  const blocked = {
    getItem(): string | null {
      throw new Error("SecurityError");
    },
  };
  assert.equal(run(blocked), undefined);
});

test("no block's state can be kept under the theme's key", () => {
  assert.ok(!THEME_KEY.startsWith("gherkin-glass:/"));
});
