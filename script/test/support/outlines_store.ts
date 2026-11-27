// The site's data of tests/fixtures/pages's outlines.feature, built with ghg.toml, as docs/BEHAVIOUR.md, "The site's data", says ghg writes it, so that the page the script draws from it can be compared with the page ghg 1 wrote, committed in script/test/expected/pages/outlines.html.

import { block, descriptor, entry, number, object, placeholder, run, step, text } from "./store_building.ts";

const HEADER = ["amount", "balance", "left", "format"];

function withdrawRun(anchor: string, amount: string, balance: string, left: string, format: string) {
  return run(anchor, [
    step("Given ", "given", [text("a balance of "), number(balance)]),
    step("When ", "when", [number(amount), text(" is withdrawn by <who>")]),
    step("Then ", "then", [text("the account shows")], { table: [["balance", amount], [left, "<none>"]] }),
    step("And ", "then", [text("the receipt reads")], { doc_string: { media_type: format, content: `Withdrawn: ${amount}` } }),
  ]);
}

const withdrawCash = block("scenario-outline", "Scenario Outline", "Withdraw cash", "withdraw-cash", {
  steps: [
    step("Given ", "given", [text("a balance of "), placeholder("<balance>")]),
    step("When ", "when", [placeholder("<amount>"), text(" is withdrawn by <who>")]),
    step("Then ", "then", [text("the account shows")], { table: [["balance", "<amount>"], ["<left>", "<none>"]] }),
    step("And ", "then", [text("the receipt reads")], { doc_string: { media_type: "<format>", content: "Withdrawn: <amount>" } }),
  ],
  examples: [
    block("examples", "Examples", "Small", "small", {
      place: 1,
      table: [HEADER, ["10", "100", "90", "text"], ["20", "100", "80", "text"]],
      runs: [withdrawRun("withdraw-cash-row-1", "10", "100", "90", "text"), withdrawRun("withdraw-cash-row-2", "20", "100", "80", "text")],
    }),
    block("examples", "Examples", "Large", "large", {
      place: 2,
      table: [HEADER, ["500", "1000", "500", "json"]],
      runs: [withdrawRun("withdraw-cash-row-3", "500", "1000", "500", "json")],
    }),
    block("examples", "Examples", "None yet", "none-yet", { place: 3, table: [HEADER] }),
  ],
});

const unnamed = block("scenario-outline", "Scenario Outline", "", "scenario-outline", {
  steps: [step("Given ", "given", [placeholder("<x>"), text(" and "), placeholder("<y>")])],
  examples: [
    block("examples", "Examples", "", "examples", {
      place: 1,
      table: [["x", "y", "x"], ["1", "<x>", "3"]],
      runs: [run("scenario-outline-row-1", [step("Given ", "given", [number("1"), text(" and <x>")])])],
    }),
  ],
});

const nothingToProduce = block("scenario-outline", "Scenario Outline", "Nothing to produce", "nothing-to-produce", {
  steps: [step("Given ", "given", [placeholder("<x>")])],
  examples: [block("examples", "Examples", "", "examples-2", { place: 1, table: [["x"]] }), block("examples", "Examples", "", "examples-3", { place: 2 })],
});

export const outlinesObject = object(
  block("feature", "Feature", "Withdraw cash at the counter", "", {
    description: "Outlines, and the scenarios their examples produce.",
    blocks: [withdrawCash, unnamed, nothingToProduce],
  }),
);

export const outlinesEntry = entry("outlines.feature", "outlines", { lines: [1, 4, 15, 19, 23, 26, 29, 33, 36, 38], source: "https://example.com/repo/blob/main/features/outlines.feature#L1" });

export const outlinesDescriptor = descriptor("Banking & <more>", [outlinesEntry]);
