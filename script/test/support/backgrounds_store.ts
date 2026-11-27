// The site's data of three feature files of tests/fixtures/backgrounds, as docs/BEHAVIOUR.md, rules 248 to 252, say ghg writes it, so that the pages the script draws from it can be compared with those ghg 1 wrote there, committed in script/test/expected/backgrounds.

import { block, descriptor, entry, number, object, placeholder, run, step, text } from "./store_building.ts";

const openingBalances = [
  step("Given ", "given", [text("an account with these balances:")], { table: [["account", "balance"], ["current", "100"]] }),
  step("And ", "given", [text("a note")], { doc_string: { media_type: "text", content: "Opened today." } }),
];

export const bothObject = object(
  block("feature", "Feature", "Transfers", "", {
    blocks: [
      block("background", "Background", "", "background", { steps: openingBalances }),
      block("scenario", "Scenario", "Outside any rule", "outside-any-rule", {
        steps: [step("When ", "when", [number("10"), text(" is moved")]), step("Then ", "then", [text("the balance is "), number("90")])],
      }),
      block("rule", "Rule", "Limits", "limits", {
        blocks: [
          block("background", "Background", "Over the limit", "over-the-limit", { steps: [step("Given ", "given", [text("a limit of "), number("50")])] }),
          block("scenario-outline", "Scenario Outline", "Move <amount>", "move-amount", {
            steps: [step("When ", "when", [placeholder("<amount>"), text(" is moved")])],
            examples: [block("examples", "Examples", "", "examples", { place: 1, table: [["amount"], ["60"]], runs: [run("move-amount-row-1", [step("When ", "when", [number("60"), text(" is moved")])])] })],
          }),
        ],
      }),
      block("rule", "Rule", "No background of its own", "no-background-of-its-own", {
        blocks: [
          block("scenario", "Scenario", "Only the feature's background", "only-the-features-background", { steps: [step("Then ", "then", [text("nothing changes")])] }),
          block("scenario", "Scenario", "Nothing of its own", "nothing-of-its-own"),
        ],
      }),
    ],
  }),
);

export const regraObject = object(
  block("feature", "Funcionalidade", "Só a regra tem contexto", "", {
    note: true,
    blocks: [
      block("scenario", "Cenário", "Fora da regra", "fora-da-regra", { note: true, steps: [step("Então ", "then", [text("nada muda")])] }),
      block("rule", "Regra", "Com contexto", "com-contexto", {
        note: true,
        blocks: [
          block("background", "Contexto", "", "contexto", { note: true, steps: [step("Dado ", "given", [text("um cliente")])] }),
          block("scenario", "Cenário", "Dentro da regra", "dentro-da-regra", { note: true, steps: [step("Então ", "then", [text("algo muda")])] }),
        ],
      }),
    ],
  }),
  "pt",
);

export const noStepsObject = object(
  block("feature", "Feature", "A background without steps", "", {
    blocks: [
      block("background", "Background", "", "background", { description: "It only describes." }),
      block("scenario", "Scenario", "Nothing to show", "nothing-to-show", { steps: [step("Then ", "then", [text("nothing comes before this step")])] }),
    ],
  }),
);

export const backgroundsEntries = { both: entry("both.feature", "both"), regra: entry("regra.feature", "regra"), noSteps: entry("no-steps.feature", "no-steps") };

export const backgroundsDescriptor = descriptor("Backgrounds", Object.values(backgroundsEntries));

export const uncheckedDescriptor = descriptor("Backgrounds", Object.values(backgroundsEntries), { start: { ...backgroundsDescriptor.start, inline_backgrounds: false } });
