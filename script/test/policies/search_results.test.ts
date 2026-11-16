// The policy of src/policies/search_results.ts, over tests/fixtures/search: docs/BEHAVIOUR.md, rules 154 to 156 and 214.

import assert from "node:assert/strict";
import { test } from "node:test";

import { allResults, moved, result, status } from "../../src/policies/search_results.ts";
import type { Result } from "../../src/ports/search_box.ts";
import { documents, pageWords } from "../support/search_fixtures.ts";

const words = pageWords();
const spoken = (found: Result): string => found.label.map((piece) => (piece.isName ? `[${piece.text}]` : piece.text)).join("");
const named = (name: string) => {
  const found = documents().find((document) => document.name === name);
  assert.ok(found, name);
  return found;
};

test("the status counts the results, and says when only the first are listed", () => {
  assert.equal(status(words, 0), "No results.");
  assert.equal(status(words, 1), "Results: 1.");
  assert.equal(status(words, 50), "Results: 50.");
  assert.equal(status(words, 51), "Results: 51. The first 50 are listed.");
});

test("each result says what it is and where, its names isolated", () => {
  assert.equal(spoken(result(words, named("Withdraw cash"))), "[Withdraw cash], feature");
  assert.equal(spoken(result(words, named("The daily limit applies"))), "[The daily limit applies], rule in [Withdraw cash]");
  assert.equal(spoken(result(words, named("Withdraw within the balance"))), "[Withdraw within the balance], scenario in [Withdraw cash]");
  assert.equal(spoken(result(words, named("Withdraw an amount"))), "[Withdraw an amount], scenario outline in [Withdraw cash]");
  assert.equal(
    spoken(result(words, named("Withdraw over the limit"))),
    "[Withdraw over the limit], scenario in [The daily limit applies], [Withdraw cash]",
  );
  const unnamed = documents().filter((document) => document.page === "banking/withdraw.html" && document.name === "");
  assert.deepEqual(
    unnamed.map((document) => spoken(result(words, document))),
    ["no name, background in [Withdraw cash]", "no name, scenario in [The daily limit applies], [Withdraw cash]"],
  );
  assert.equal(spoken(result(words, named("Limits in force"))), "[Limits in force], background in [The daily limit applies], [Withdraw cash]");
  assert.equal(spoken(result(words, named("Small amounts"))), "[Small amounts], examples in [Withdraw an amount], [Withdraw cash]");
});

test("a feature or rule without a name reads as one, an unnamed examples block is numbered, and a block in a rule says so", () => {
  const page = documents().filter((document) => document.page === "unnamed.html");
  assert.deepEqual(
    page.map((document) => spoken(result(words, document))),
    [
      "no name, feature",
      "no name, rule in no name",
      "[Pay <amount>], scenario outline in no name, no name",
      "Examples 1, examples in [Pay <amount>], no name, no name",
      "Examples 2, examples in [Pay <amount>], no name, no name",
      "[Nothing yet], rule in no name",
    ],
  );
});

test("each result leads to its page and anchor, encoded", () => {
  assert.equal(result(words, named("Withdraw cash")).address, "banking/withdraw.html");
  assert.equal(result(words, named("Withdraw over the limit")).address, "banking/withdraw.html#withdraw-over-the-limit");
  assert.equal(result(words, named("Saque com saldo")).address, "saque.html#saque-com-saldo");
  const odd = { ...named("Saque com saldo"), page: "a b/c#d.html", anchor: "poupança" };
  assert.equal(result(words, odd).address, "a%20b/c%23d.html#poupan%C3%A7a");
});

test("the link to every result is there only for words that are not only spaces", () => {
  assert.equal(allResults(words, "   "), null);
  const link = allResults(words, " books & films ");
  assert.equal(link?.address, "search.html?q=books+%26+films");
  assert.equal(link?.label.map((piece) => (piece.isName ? `[${piece.text}]` : piece.text)).join(""), 'All results for "[books & films]"');
});

test("the arrows move from none to either end, and past either end to none", () => {
  assert.equal(moved(null, 3, 1), 0);
  assert.equal(moved(null, 3, -1), 2);
  assert.equal(moved(1, 3, 1), 2);
  assert.equal(moved(2, 3, 1), null);
  assert.equal(moved(0, 3, -1), null);
  assert.equal(moved(null, 0, 1), null);
});
