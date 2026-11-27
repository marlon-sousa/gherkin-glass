// The site's data of tests/fixtures/step-sentences, each step with only its kind, its sentence and its values, so that the page of step sentences the script draws from it can be compared with the one ghg 1 wrote.

import { block, descriptor, entry, object, step, text } from "./store_building.ts";
import type { LoadedFeature } from "../../src/policies/site_folders.ts";
import type { Step, StepKind } from "../../src/ports/store.ts";

const used = (kind: StepKind, sentence: string, ...slots: string[]): Step => step("* ", kind, [text(sentence)], { sentence, slots });

const loggingIn = object(
  block("feature", "Feature", "Logging in", "", {
    blocks: [
      block("background", "Background", "", "background", { steps: [used("given", 'the user "…" exists', '"ana"')] }),
      block("scenario", "Scenario", "Ana logs in", "ana-logs-in", {
        steps: [used("given", 'the user "…" logs in', '"ana"'), used("when", "she opens her account"), used("then", 'she sees "…"', '"Welcome"'), used("then", 'she sees "…"', '"Welcome"')],
      }),
      block("scenario", "Scenario", "Bia logs in", "bia-logs-in", { steps: [used("given", 'the user "…" logs in', '"bia"'), used("then", 'She sees "…"', '"Welcome"')] }),
      block("scenario-outline", "Scenario Outline", "Anyone logs in", "anyone-logs-in", {
        steps: [used("given", 'the user "…" logs in', "<name>"), used("given", 'the user "…" exists', '"<name>"'), used("when", 'I withdraw "…"', "<amount>"), used("then", "the balance is < 0 and <unknown>")],
      }),
    ],
  }),
);

const accountsOld = object(
  block("feature", "Feature", "Old accounts", "", {
    blocks: [
      block("scenario", "Scenario", "Star steps", "star-steps", {
        steps: [
          used("given", 'the user "…" exists', '"fay"'),
          used("when", "the old account is opened"),
          used("then", "it is shown"),
          used("given", "a star step"),
          used("given", "a step after a star"),
          used("given", "a step with a table:"),
          used("given", "a step with a doc string:"),
        ],
      }),
      block("scenario-outline", "Scenario Outline", "Two examples", "two-examples", { steps: [used("given", 'the user "…" and "…" meet', "<first>", "<second>")] }),
    ],
  }),
);

const emPortugues = object(
  block("feature", "Funcionalidade", "Entrar", "", {
    blocks: [
      block("scenario", "Cenário", "Ana entra", "ana-entra", {
        steps: [used("given", 'o usuário "…" entra', '"ana"'), used("when", "ela abre a conta"), used("then", 'ela vê "…"', '"Bem-vinda"'), used("then", 'ela vê "…"', '"Bem-vinda"'), used("then", 'ela não vê "…"', '"Erro"')],
      }),
      block("scenario", "Cenário", "Gil", "gil", { steps: [used("given", 'the user "…" exists', '"gil"')] }),
    ],
  }),
  "pt",
);

const escapes = object(
  block("feature", "Feature", "Escapes", "", {
    blocks: [
      block("scenario-outline", "Scenario Outline", "Quotes after a backslash", "quotes-after-a-backslash", {
        steps: [used("then", 'the screen shows \\"hi\\" and "…" for "…"', '"a \\" b"', "<name>"), used("then", 'the screen shows <> for "…"', "<name>")],
      }),
    ],
  }),
);

const money = object(
  block("feature", "Feature", "Money", "", {
    blocks: [
      block("scenario", "Scenario", "Withdraw", "withdraw", {
        steps: [
          used("when", "I withdraw 10"),
          used("when", "I withdraw 20"),
          used("then", 'the note says "…" and "…"', '""', '"it\'s done"'),
          used("then", 'the user\'s card is "kept'),
          used("then", "the screen shows “hi”"),
          used("then", "the café is paid"),
        ],
      }),
      block("scenario", "Scenario", "Starting with and", "starting-with-and", { steps: [used("given", "the first step is an and"), used("then", "the café is paid")] }),
      block("rule", "Rule", "Every user", "every-user", {
        blocks: [
          block("background", "Background", "", "background", { steps: [used("given", 'the user "…" exists', '"dan"')] }),
          block("scenario", "Scenario", "Eve logs in", "eve-logs-in", { steps: [used("when", 'the user "…" logs in', '"eve"'), used("then", "the user <name> logs in")] }),
        ],
      }),
    ],
  }),
);

export const sentencesFeatures: readonly LoadedFeature[] = [
  { entry: entry("accounts/logging-in.feature", "logging-in"), object: loggingIn },
  { entry: entry("accounts-old.feature", "accounts-old"), object: accountsOld },
  { entry: entry("em-portugues.feature", "em-portugues"), object: emPortugues },
  { entry: entry("escapes.feature", "escapes"), object: escapes },
  { entry: entry("money.feature", "money"), object: money },
];

export const sentencesDescriptor = descriptor(
  "Fixture",
  sentencesFeatures.map((feature) => feature.entry),
  {
    sentences: {
      given: ["a star step", "a step after a star", "a step with a doc string:", "a step with a table:", 'o usuário "…" entra', "the first step is an and", 'the user "…" and "…" meet', 'the user "…" exists', 'the user "…" logs in'],
      when: ["ela abre a conta", 'I withdraw "…"', "I withdraw 10", "I withdraw 20", "she opens her account", "the old account is opened", 'the user "…" logs in'],
      then: [
        'ela não vê "…"',
        'ela vê "…"',
        "it is shown",
        'She sees "…"',
        'she sees "…"',
        "the balance is < 0 and <unknown>",
        "the café is paid",
        'the note says "…" and "…"',
        'the screen shows <> for "…"',
        'the screen shows \\"hi\\" and "…" for "…"',
        "the screen shows “hi”",
        "the user <name> logs in",
        'the user\'s card is "kept',
      ],
    },
  },
);
