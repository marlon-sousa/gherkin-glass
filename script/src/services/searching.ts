// Service: searching the site's documents from the search box, a combobox, the index built the first time a search is typed.

import { PAUSE_MILLISECONDS, allResults, moved, result, status } from "../policies/search_results.ts";
import { type Index, type SearchAddresses, WRITTEN_ADDRESSES, indexOf } from "../policies/search_index.ts";
import type { Documents } from "../ports/documents.ts";
import type { Result, SearchBox } from "../ports/search_box.ts";
import type { Timer } from "../ports/timer.ts";

/** Puts the words of the page of search results in the field, its list closed until the reader types. */
export type SearchFilling = (text: string) => void;

export function startSearching(box: SearchBox, documents: Documents, timer: Timer, addresses: SearchAddresses = WRITTEN_ADDRESSES): SearchFilling {
  const words = box.words();
  let index: Index | null = null;
  let loading: Promise<Index | null> | null = null;
  // The words the field already holds, so that its link and Enter work before the reader types.
  let text = box.fieldText();
  let started = false;
  let results: readonly Result[] = [];
  let chosen: number | null = null;
  let open = false;
  /** The status of the last search, until the reader has paused. */
  let pending: string | null = null;
  let paused = false;

  /** Null when the documents cannot be fetched or read; the next call tries again. */
  const load = (): Promise<Index | null> => {
    if (index !== null) {
      return Promise.resolve(index);
    }
    loading ??= documents
      .load()
      .then((found) => {
        index = indexOf(found);
        return index;
      })
      .catch(() => {
        loading = null;
        return null;
      });
    return loading;
  };

  const announce = (): void => {
    if (paused && pending !== null) {
      box.setStatus(pending);
      pending = null;
    }
  };

  const show = (shown: readonly Result[]): void => {
    results = shown;
    chosen = null;
    box.showResults(results);
    setOpen(results.length > 0);
  };

  const setOpen = (opened: boolean): void => {
    open = opened;
    box.setOpen(opened);
  };

  /** Shows the results of the text in the field, once the documents are loaded. */
  const run = async (): Promise<void> => {
    const asked = text;
    if (index === null) {
      box.setStatus(words.loading);
    }
    const found = await load();
    if (asked !== text) {
      return;
    }
    if (found === null) {
      show([]);
      pending = words.failed;
    } else {
      const matches = found.search(asked);
      show(matches.slice(0, words.shown).map((match) => result(words, match.document, addresses)));
      pending = status(words, matches.length);
    }
    announce();
  };

  const empty = (): void => {
    timer.cancel();
    pending = null;
    show([]);
    box.setStatus("");
  };

  box.onInput((typed) => {
    text = typed;
    box.showAllResults(allResults(words, typed, addresses));
    if (typed.trim() === "") {
      empty();
      return;
    }
    // Fetching starts with the first letter typed, while the reader goes on typing.
    if (!started) {
      started = true;
      void load();
    }
    paused = false;
    timer.after(PAUSE_MILLISECONDS, () => {
      paused = true;
      announce();
    });
    void run();
  });
  const move = (step: 1 | -1): void => {
    if (!open) {
      if (results.length === 0) {
        return;
      }
      setOpen(true);
    }
    chosen = moved(chosen, results.length, step);
    box.choose(chosen);
  };
  box.showAllResults(allResults(words, text, addresses));
  box.onDown(() => move(1));
  box.onUp(() => move(-1));
  box.onEnter(() => {
    const picked = open && chosen !== null ? results[chosen] : undefined;
    const target = picked ?? allResults(words, text, addresses);
    if (target !== null) {
      box.go(target.address);
    }
  });
  box.onPick((place) => {
    const picked = results[place];
    if (picked !== undefined) {
      box.go(picked.address);
    }
  });
  box.onEscape(() => {
    if (open) {
      chosen = null;
      box.choose(null);
      setOpen(false);
      return;
    }
    text = "";
    box.fill("");
    box.showAllResults(null);
    empty();
  });
  return (filled) => {
    text = filled;
    box.fill(filled);
    box.showAllResults(allResults(words, filled, addresses));
    empty();
  };
}
