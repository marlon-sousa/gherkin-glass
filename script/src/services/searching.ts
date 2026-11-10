// Service: searching the site's documents from the search box, the index built the first time a search is typed.

import MiniSearch from "minisearch";

import { INDEX_OPTIONS, type Indexed, PAUSE_MILLISECONDS, SEARCH_OPTIONS, result, status } from "../policies/search_results.ts";
import type { Documents, SearchDocument } from "../ports/documents.ts";
import type { SearchBox } from "../ports/search_box.ts";
import type { Timer } from "../ports/timer.ts";

interface Index {
  readonly documents: readonly SearchDocument[];
  readonly search: MiniSearch<Indexed>;
}

export function startSearching(box: SearchBox, documents: Documents, timer: Timer): void {
  const words = box.words();
  let index: Index | null = null;
  let loading: Promise<Index | null> | null = null;
  let text = "";
  let started = false;

  /** Null when the documents cannot be fetched or read; the next call tries again. */
  const load = (): Promise<Index | null> => {
    if (index !== null) {
      return Promise.resolve(index);
    }
    loading ??= documents
      .load()
      .then((found) => {
        const search = new MiniSearch<Indexed>(INDEX_OPTIONS);
        search.addAll(found.map((document, id) => ({ ...document, id })));
        index = { documents: found, search };
        return index;
      })
      .catch(() => {
        loading = null;
        return null;
      });
    return loading;
  };

  /** True when it showed the results of the text still in the field. */
  const run = async (): Promise<boolean> => {
    const asked = text;
    if (asked.trim() === "") {
      empty();
      return false;
    }
    if (index === null) {
      box.setStatus(words.loading);
    }
    const found = await load();
    if (asked !== text) {
      return false;
    }
    if (found === null) {
      box.showResults([]);
      box.setStatus(words.failed);
      return false;
    }
    const matches = found.search.search(asked, SEARCH_OPTIONS);
    box.showResults(
      matches.slice(0, words.shown).map((match) => result(words, found.documents[match.id as number] as SearchDocument)),
    );
    box.setStatus(status(words, matches.length));
    return true;
  };

  const empty = (): void => {
    timer.cancel();
    box.showResults([]);
    box.setStatus("");
  };

  box.onInput((typed) => {
    text = typed;
    if (typed.trim() === "") {
      empty();
      return;
    }
    // Fetching starts with the first letter typed, while the reader goes on typing.
    if (!started) {
      started = true;
      void load();
    }
    timer.after(PAUSE_MILLISECONDS, () => void run());
  });
  box.onEnter(() => {
    timer.cancel();
    void run().then((shown) => {
      if (shown) {
        box.focusFirstResult();
      }
    });
  });
  box.onEscape(() => {
    text = "";
    box.clear();
    empty();
  });
}
