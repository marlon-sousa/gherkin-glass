// The site's data of tests/fixtures/pages's billing/refunds.feature, built with ghg.toml, as docs/BEHAVIOUR.md, rules 248 to 252, say ghg writes it, so that the page the script draws from it can be compared with the page ghg 1 wrote, committed in script/test/expected/pages/billing/refunds.html.

import { block, descriptor, entry, number, object, parameter, placeholder, run, step, tag, text } from "./store_building.ts";

const customer = step("Given ", "given", [text("a customer named "), parameter(text('"Ana"'))], { sentence: 'a customer named "…"', slots: ['"Ana"'] });

const cardPayment = block("scenario", "Scenario", "Refund a card payment", "refund-a-card-payment", {
  description: "The plain case.",
  steps: [
    step("Given ", "given", [text("an order with these lines:")], { table: [["item", "price"], ["pen", "1.50"], ["a | b", "2"]] }),
    step("When ", "when", [text("the order is refunded")]),
    step("Then ", "then", [text("the card gets")], { doc_string: { media_type: "json", content: '{"amount": 3.50}' } }),
    step("And ", "then", [text("a note is kept")], { doc_string: { media_type: null, content: "\nRefunded in full.\n  Indented." } }),
  ],
});

const countItems = block("scenario-outline", "Scenario Outline", "Refund <count> items", "refund-count-items", {
  tags: ["@outline"],
  steps: [step("Given ", "given", [placeholder("<count>"), text(" items")]), step("Then ", "then", [text("the total is "), placeholder("<total>")])],
  examples: [
    block("examples", "Examples", "Cheap", "cheap", {
      tags: ["@cheap"],
      place: 1,
      table: [["count", "total"], ["1", "2"]],
      runs: [run("refund-count-items-row-1", [step("Given ", "given", [number("1"), text(" items")]), step("Then ", "then", [text("the total is "), number("2")])])],
    }),
    block("examples", "Examples", "", "examples", { place: 2, table: [["count", "total"]] }),
  ],
});

const afterThirtyDays = block("rule", "Rule", "Refunds after thirty days", "refunds-after-thirty-days", {
  description: "Only store credit.",
  blocks: [
    block("background", "Background", "An old order", "an-old-order", { steps: [step("Given ", "given", [text("an order from forty days ago")])] }),
    block("scenario", "Example", "Store credit", "store-credit", { note: true, tags: ["@in-rule"], steps: [step("Then ", "then", [text("store credit is given")])] }),
    block("scenario-outline", "Scenario Outline", "Refund again <count> times", "refund-again-count-times", {
      steps: [step("Given ", "given", [placeholder("<count>"), text(" refunds")])],
      examples: [
        block("examples", "Examples", "Again", "again", {
          place: 1,
          table: [["count"], ["2"]],
          runs: [run("refund-again-count-times-row-1", [step("Given ", "given", [number("2"), text(" refunds")])])],
        }),
      ],
    }),
  ],
});

const unnamedRule = block("rule", "Rule", "", "rule", { blocks: [block("scenario", "Scenario", "Refund a refund", "refund-a-refund", { steps: [step("Then ", "then", [text("it is refused")])] })] });

export const refundsObject = object(
  block("feature", "Feature", "Refunds & <returns>", "", {
    description: "A refund gives the money back,\nin the way it was paid.",
    tags: ["@billing", "@slow"],
    blocks: [block("background", "Background", "", "background", { steps: [customer] }), cardPayment, countItems, afterThirtyDays, unnamedRule],
  }),
);

export const refundsEntry = entry("billing/refunds.feature", "refunds", { source: "https://example.com/repo/blob/main/features/billing/refunds.feature#L2" });

export const refundsTags = [tag("@billing", "tags/billing"), tag("@cheap", "tags/cheap"), tag("@in-rule", "tags/in-rule"), tag("@outline", "tags/outline"), tag("@slow", "tags/slow")];

export const refundsDescriptor = descriptor("Banking & <more>", [refundsEntry], { tags: refundsTags });
