// The adapter of src/adapters/media_width.ts, over a media query: docs/BEHAVIOUR.md, rule 87.

import assert from "node:assert/strict";
import { test } from "node:test";

import { MediaWidth } from "../../src/adapters/media_width.ts";

class FakeQuery extends EventTarget {
  matches = false;
  resize(matches: boolean): void {
    this.matches = matches;
    this.dispatchEvent(Object.assign(new Event("change"), { matches }));
  }
}

test("the window is narrow while the query matches, and says when that changes", () => {
  const query = new FakeQuery();
  const width = new MediaWidth(query as unknown as MediaQueryList);
  const heard: boolean[] = [];
  width.onChange((narrow) => heard.push(narrow));
  assert.equal(width.narrow(), false);
  query.resize(true);
  assert.equal(width.narrow(), true);
  query.resize(false);
  assert.deepEqual(heard, [true, false]);
});
