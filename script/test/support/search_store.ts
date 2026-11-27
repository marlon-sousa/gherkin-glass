// The site's data of tests/fixtures/search, each step with only its text, so that the documents the script makes from it can be compared with those ghg 1 wrote, tests/fixtures/search/search.json.

import { block, entry, object, step, text } from "./store_building.ts";
import type { LoadedFeature } from "../../src/policies/site_folders.ts";
import type { Step } from "../../src/ports/store.ts";

const steps = (...texts: string[]): Step[] => texts.map((words) => step("* ", "given", [text(words)]));

const withdraw = object(
  block("feature", "Feature", "Withdraw cash", "", {
    tags: ["@banking", "@cash"],
    description: "Customers take money out of their accounts.",
    blocks: [
      block("background", "Background", "", "background", { steps: steps("the customer has an account") }),
      block("scenario", "Scenario", "Withdraw within the balance", "withdraw-within-the-balance", { tags: ["@smoke"], steps: steps("the balance is 100", "the customer withdraws 40", "the receipt says") }),
      block("scenario-outline", "Scenario Outline", "Withdraw an amount", "withdraw-an-amount", {
        steps: steps("the customer withdraws <amount>", "the balance is <balance>"),
        examples: [
          block("examples", "Examples", "Small amounts", "small-amounts", { tags: ["@regression"], place: 1, table: [["amount", "balance"], ["10", "90"]] }),
          block("examples", "Examples", "Large amounts", "large-amounts", { tags: ["@regression", "@slow"], place: 2, table: [["amount", "balance"], ["90", "10"]] }),
        ],
      }),
      block("rule", "Rule", "The daily limit applies", "the-daily-limit-applies", {
        tags: ["@limits"],
        blocks: [
          block("background", "Background", "Limits in force", "limits-in-force", { steps: steps("the daily limit is 500") }),
          block("scenario", "Scenario", "Withdraw over the limit", "withdraw-over-the-limit", { steps: steps("the customer withdraws 600", "the withdrawal is refused") }),
          block("scenario", "Scenario", "", "scenario", { steps: steps("the customer withdraws 500", "the withdrawal is accepted") }),
        ],
      }),
    ],
  }),
);

const saque = object(
  block("feature", "Funcionalidade", "Saque em conta poupança", "", {
    blocks: [block("scenario", "Cenário", "Saque com saldo", "saque-com-saldo", { steps: steps("que o saldo é 100", "o cliente saca 40", "o saldo é 60") })],
  }),
  "pt",
);

const unnamed = object(
  block("feature", "Feature", "", "", {
    blocks: [
      block("rule", "Rule", "", "rule", {
        blocks: [
          block("scenario-outline", "Scenario Outline", "Pay <amount>", "pay-amount", {
            tags: ["@tagged"],
            steps: steps("the customer pays <amount>"),
            examples: [
              block("examples", "Examples", "", "examples", { tags: ["@tagged", "@paid"], place: 1, table: [["amount"], ["10"]] }),
              block("examples", "Examples", "", "examples-2", { place: 2, table: [["amount", "payer"], ["20", "Emma"]] }),
            ],
          }),
        ],
      }),
      block("rule", "Rule", "Nothing yet", "nothing-yet"),
    ],
  }),
);

export const searchFeatures: readonly LoadedFeature[] = [
  { entry: entry("banking/withdraw.feature", "withdraw"), object: withdraw },
  { entry: entry("saque.feature", "saque"), object: saque },
  { entry: entry("unnamed.feature", "unnamed"), object: unnamed },
];
