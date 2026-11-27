// The adapter of src/adapters/dom_address.ts, on a shell in a document.

import assert from "node:assert/strict";
import { test } from "node:test";

import { Window } from "happy-dom";

import { DomAddress } from "../../src/adapters/dom_address.ts";

// Rule 236: a link to an element of the page shown is caught, leaving the address as it was.
test("a link written ## is caught, and the address stays as it was", () => {
  const window = new Window({ url: "https://example.com/site/v1/index.html#billing/" });
  window.document.write('<!doctype html><html lang="en"><body><a class="ghg-skip" href="##ghg_main">Skip to content</a><main id="ghg_main"></main></body></html>');
  const followed: string[] = [];
  new DomAddress(window as unknown as globalThis.Window).onChange((hash) => followed.push(hash));
  (window.document.querySelector("a.ghg-skip") as unknown as HTMLElement).click();
  assert.deepEqual(followed, ["##ghg_main"]);
  assert.equal(window.location.hash, "#billing/");
});
