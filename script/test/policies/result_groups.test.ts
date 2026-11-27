// The policy of src/policies/result_groups.ts, over tests/fixtures/search: docs/BEHAVIOUR.md, rules 216 and 218 to 220.

import assert from "node:assert/strict";
import { test } from "node:test";

import { groupedResults, pageStatus, searchedHeading } from "../../src/policies/result_groups.ts";
import { indexOf } from "../../src/policies/search_index.ts";
import type { Part, Run } from "../../src/ports/search_page.ts";
import { documents, searchPageWords } from "../support/search_fixtures.ts";

const words = searchPageWords();
const runs = (text: readonly Run[]): string => text.map((run) => (run.marked ? `*${run.text}*` : run.text)).join("");
const spoken = (parts: readonly Part[] | null): string =>
  (parts ?? []).map((part) => (part.isName ? `[${runs(part.runs)}]` : runs(part.runs))).join("");
const grouped = (text: string) => groupedResults(words, indexOf(documents()).search(text));

test("the heading and the title name the words searched for, isolated in the heading", () => {
  const { heading, title } = searchedHeading(words, "limit refused");
  assert.equal(spoken(heading), 'Results for "[limit refused]"');
  assert.equal(title, 'Results for "limit refused", Banking & <more>');
});

test("words that look like a replacement pattern are kept as written in the title", () => {
  const { heading, title } = searchedHeading(words, "$&x");
  assert.equal(spoken(heading), 'Results for "[$&x]"');
  assert.equal(title, 'Results for "$&x", Banking & <more>');
});

test("the results are grouped by feature, in the order of each group's best result", () => {
  const groups = grouped("withdraw");
  assert.deepEqual(
    groups.map((group) => [spoken(group.heading), group.address]),
    [["Feature: [Withdraw cash]", "#banking/withdraw"]],
  );
  const mixed = grouped("amount");
  assert.deepEqual(
    mixed.map((group) => spoken(group.heading)),
    ["Feature: [Withdraw cash]", "Feature"],
  );
});

test("each result reads its block's heading and the blocks it is in, as on its page", () => {
  const [group] = grouped("emma");
  const [result] = group?.results ?? [];
  assert.equal(spoken(result?.heading ?? []), "Examples 2");
  assert.equal(result?.address, "#unnamed#examples-2");
  assert.equal(spoken(result?.within ?? null), "In Rule, Scenario outline: [Pay <amount>].");
  const [limits] = grouped("limits in force")[0]?.results ?? [];
  assert.equal(spoken(limits?.heading ?? []), "Background: [*Limits* *in* *force*]");
  assert.equal(spoken(limits?.within ?? null), "In Rule: [The daily limit applies].");
  const [feature] = grouped("@cash")[0]?.results ?? [];
  assert.equal(feature?.address, "#banking/withdraw");
  assert.equal(feature?.within, null);
});

test("only the steps, tags and cells the search matched are shown, their matched words marked", () => {
  const [result] = grouped("refused")[0]?.results ?? [];
  assert.deepEqual(result?.steps.map(runs), ["the withdrawal is *refused*"]);
  assert.deepEqual(result?.tags, []);
  const [emma] = grouped("emma")[0]?.results ?? [];
  assert.deepEqual(emma?.cells.map(runs), ["*Emma*"]);
  const [slow] = grouped("slow")[0]?.results ?? [];
  assert.deepEqual(slow?.tags.map(runs), ["@*slow*"]);
});

test("the status counts the results and the features they are in", () => {
  assert.equal(pageStatus(words, grouped("zebra")), "No results.");
  const groups = grouped("amount");
  const count = groups.reduce((sum, group) => sum + group.results.length, 0);
  assert.equal(pageStatus(words, groups), `Results: ${count}. Features: 2.`);
});
