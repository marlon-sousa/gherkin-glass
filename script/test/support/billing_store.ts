// The site's data of tests/fixtures/pages's billing folder, its refunds and its archive's closed accounts, built with ghg.toml, so that its folder page and its part of the folder tree can be compared with those ghg 1 wrote.

import { refundsEntry, refundsObject, refundsTags } from "./refunds_store.ts";
import { block, descriptor, entry, object } from "./store_building.ts";
import type { LoadedFeature } from "../../src/policies/site_folders.ts";

export const closedObject = object(block("feature", "Feature", "Closed accounts", "", { blocks: [block("rule", "Rule", "No refunds once closed", "no-refunds-once-closed")] }));

export const closedEntry = entry("billing/archive/closed #1.feature", "closed-1");

/** In the descriptor's order, the archive's file first, as its folder is. */
export const billingFeatures: readonly LoadedFeature[] = [
  { entry: closedEntry, object: closedObject },
  { entry: refundsEntry, object: refundsObject },
];

export const billingDescriptor = descriptor(
  "Banking & <more>",
  billingFeatures.map((feature) => feature.entry),
  { tags: refundsTags },
);
