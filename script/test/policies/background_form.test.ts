// The policy of src/policies/background_form.ts.

import assert from "node:assert/strict";
import { test } from "node:test";

import { formKey, rememberedForm, siteRoot, startForm } from "../../src/policies/background_form.ts";

test("a remembered form wins over the configured one", () => {
  assert.equal(startForm(true, "linked"), false);
  assert.equal(startForm(false, "inline"), true);
  assert.equal(startForm(false, null), false);
  assert.equal(startForm(true, "something else"), true);
  assert.equal(startForm(true, rememberedForm(false)), false);
  assert.equal(startForm(false, rememberedForm(true)), true);
});

test("the site's root is the folder above the script's", () => {
  assert.equal(siteRoot("https://example.com/repo/assets/gherkin-glass.js"), "/repo/");
  assert.equal(siteRoot("https://example.com/assets/gherkin-glass.js"), "/");
  assert.equal(siteRoot("http://localhost:8000/v1.0/assets/gherkin-glass.js?x"), "/v1.0/");
});

test("the form is kept by site, apart from every block's state", () => {
  assert.equal(formKey("/repo/"), "gherkin-glass:/repo/:backgrounds");
  assert.notEqual(formKey("/a/"), formKey("/b/"));
  assert.ok(!formKey("/repo/").includes("#"));
});
