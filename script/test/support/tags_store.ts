// The site's data of tests/fixtures/tags, built with its ghg.toml, so that the tag pages the script draws from it can be compared with those ghg 1 wrote.

import { block, descriptor, entry, object, step, tag, text } from "./store_building.ts";
import type { LoadedFeature } from "../../src/policies/site_folders.ts";

const accounts = object(
  block("feature", "Feature", "Accounts", "", {
    tags: ["@Slow", "@slow", "@index"],
    description: "Tags that share their name once lower-cased, and one named as the index.",
    blocks: [
      block("scenario", "Scenario", "Open an account", "open-an-account", { tags: ["@issue-12", "@needs:db"], steps: [step("Given ", "given", [text("a customer")])] }),
      block("scenario-outline", "Scenario Outline", "Close <count> accounts", "close-count-accounts", {
        examples: [
          block("examples", "Examples", "Many", "many", { tags: ["@ticket:ç/1#2"], place: 1, table: [["count"], ["2"]] }),
          block("examples", "Examples", "None", "none", { place: 2, table: [["count"]] }),
        ],
      }),
      block("rule", "Rule", "Joint accounts", "joint-accounts", {
        tags: ["@issue-7"],
        blocks: [block("scenario", "Scenario", "Open a joint account", "open-a-joint-account", { tags: ["@issue-12"] }), block("scenario", "Scenario", "Close a joint account", "close-a-joint-account")],
      }),
      block("rule", "Rule", "Nothing yet", "nothing-yet", { tags: ["@needs:db"] }),
    ],
  }),
);

const notes = object(block("feature", "Feature", "Notes with no scenarios", "", { tags: ["@draft", "@later"] }));

const saque = object(
  block("feature", "Funcionalidade", "Saque em conta", "", {
    tags: ["@needs:db", "@🚀", "@???"],
    blocks: [block("scenario", "Cenário", "Saque no caixa", "saque-no-caixa", { tags: ["@issue-12"] })],
  }),
  "pt",
);

export const tagsFeatures: readonly LoadedFeature[] = [
  { entry: entry("accounts.feature", "accounts"), object: accounts },
  { entry: entry("area/notes.feature", "notes"), object: notes },
  { entry: entry("area/saque em conta.feature", "saque-em-conta"), object: saque },
];

export const tagsDescriptor = descriptor(
  "Tagged suite",
  tagsFeatures.map((feature) => feature.entry),
  {
    tags: [
      tag("@???", "tags/tag"),
      tag("@draft", "tags/draft"),
      tag("@index", "tags/index-2"),
      tag("@issue-12", "tags/issue-12", { address: "https://github.com/owner/repo/issues/12", text: "Issue 12 on GitHub" }),
      tag("@issue-7", "tags/issue-7", { address: "https://github.com/owner/repo/issues/7", text: "Issue 7 on GitHub" }),
      tag("@later", "tags/later"),
      tag("@needs:db", "tags/needs-db"),
      tag("@Slow", "tags/slow"),
      tag("@slow", "tags/slow-2"),
      tag("@ticket:ç/1#2", "tags/ticket-ç-1-2", { address: "https://example.com/tickets/%C3%A7/1%232?view=$full", text: "Ticket ç/1#2" }),
      tag("@🚀", "tags/tag-2"),
    ],
  },
);
