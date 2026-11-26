// The adapter of src/adapters/fetched_store.ts, with a fake of `fetch`.

import assert from "node:assert/strict";
import { test } from "node:test";

import { FetchedStore } from "../../src/adapters/fetched_store.ts";
import { outlinesDescriptor, outlinesObject } from "../support/outlines_store.ts";

/** A `fetch` that answers the files of `files` by their address, counting what it is asked for. */
function fetcher(files: Map<string, unknown>, asked: string[]): typeof fetch {
  return async (input) => {
    const address = String(input);
    asked.push(address);
    const file = files.get(address);
    return file === undefined ? new Response("", { status: 404 }) : new Response(JSON.stringify(file));
  };
}

const ROOT = new URL("https://example.com/site/data/");

// Rule 241: the descriptor the shell names is fetched from the store.
test("the descriptor the shell names, and each object, are fetched from the store", async () => {
  const asked: string[] = [];
  const files = new Map<string, unknown>([
    ["https://example.com/site/data/versions/v1.0.0.json", outlinesDescriptor],
    ["https://example.com/site/data/objects/3f.json", outlinesObject],
  ]);
  const store = new FetchedStore(ROOT, "versions/v1.0.0.json", fetcher(files, asked));
  assert.deepEqual(await store.descriptor(), outlinesDescriptor);
  assert.deepEqual(await store.object("3f"), outlinesObject);
  await store.object("3f");
  assert.deepEqual(asked, ["https://example.com/site/data/versions/v1.0.0.json", "https://example.com/site/data/objects/3f.json"]);
});

// Rules 152 and 243: a file that cannot be fetched rejects, and is asked for again next time.
test("a file that cannot be fetched or read rejects, and is fetched again when asked again", async () => {
  const asked: string[] = [];
  const files = new Map<string, unknown>([["https://example.com/site/data/pages.json", { format: 1 }]]);
  const store = new FetchedStore(ROOT, "site.json", fetcher(files, asked));
  await assert.rejects(store.versions());
  await assert.rejects(store.versions());
  await assert.rejects(store.pages(), TypeError);
  assert.deepEqual(asked, ["https://example.com/site/data/versions.json", "https://example.com/site/data/versions.json", "https://example.com/site/data/pages.json"]);
});
