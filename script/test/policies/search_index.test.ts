// The policy of src/policies/search_index.ts, over tests/fixtures/search: docs/BEHAVIOUR.md, rules 153 and 220.

import assert from "node:assert/strict";
import { test } from "node:test";

import { address, holdsMatch, indexOf, marked, resultsPage } from "../../src/policies/search_index.ts";
import { documents } from "../support/search_fixtures.ts";

const shown = (runs: ReturnType<typeof marked>): string => runs.map((run) => (run.marked ? `[${run.text}]` : run.text)).join("");

test("each field the search matched names the words it matched in it", () => {
  const [found] = indexOf(documents()).search("withdrawl refus");
  assert.equal(found?.document.name, "Withdraw over the limit");
  assert.deepEqual([...(found?.matched.get("name") ?? [])], ["withdraw"]);
  assert.deepEqual([...(found?.matched.get("steps") ?? [])].sort(), ["refused", "withdrawal", "withdraws"]);
});

test("the words matched are marked, compared lower-cased, and the rest kept as written", () => {
  assert.equal(shown(marked("The customer Withdraws 600, then: refused!", new Set(["withdraws", "refused"]))), "The customer [Withdraws] 600, then: [refused]!");
  assert.equal(shown(marked("@regression", new Set(["regression"]))), "@[regression]");
  assert.equal(shown(marked("nothing here", undefined)), "nothing here");
  assert.equal(holdsMatch(marked("nothing here", new Set(["else"]))), false);
});

test("an entry's address and the page of every result are encoded", () => {
  const odd = { ...documents()[0]!, page: "a b/c#d.html", anchor: "poupança" };
  assert.equal(address(odd), "a%20b/c%23d.html#poupan%C3%A7a");
  assert.equal(resultsPage("ação & co"), "search.html?q=a%C3%A7%C3%A3o+%26+co");
});
