// Service: listing on the page of search results everything found for the words of its address, grouped by feature.

import { groupedResults, pageStatus, searchedHeading } from "../policies/result_groups.ts";
import { indexOf } from "../policies/search_index.ts";
import type { Documents } from "../ports/documents.ts";
import type { SearchPage } from "../ports/search_page.ts";

/** Settles once the results, or the status that they cannot be had, are shown. */
export async function startListing(page: SearchPage, documents: Documents): Promise<void> {
  const words = page.words();
  const text = page.searched().trim();
  if (text === "") {
    return;
  }
  const { heading, title } = searchedHeading(words, text);
  page.showSearched(text, heading, title);
  page.setStatus(words.loading);
  let groups;
  try {
    groups = groupedResults(words, indexOf(await documents.load()).search(text));
  } catch {
    page.setStatus(words.failed);
    return;
  }
  page.showGroups(groups);
  page.setStatus(pageStatus(words, groups));
}
