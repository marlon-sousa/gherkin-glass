// Port: each kind of page, drawn from its view in the shell's `main`.

import type { FeatureView } from "../policies/feature_view.ts";
import type { FolderView } from "../policies/folder_view.ts";
import type { PageInVersions } from "../policies/other_versions.ts";
import type { SearchPageView } from "../policies/search_page_view.ts";
import type { SentencesView } from "../policies/sentences_view.ts";
import type { TagIndexView, TagPageView } from "../policies/tag_views.ts";

/** The page of rule 239, or the paragraph of rule 243. */
export interface NoticeView {
  readonly heading: string;
  readonly text: string;
}

export interface Pages {
  /** `versions` holds its lists in other versions, null on a site without versions or when they cannot be had. */
  feature(view: FeatureView, versions: PageInVersions | null): void;
  folder(view: FolderView): void;
  tagIndex(view: TagIndexView): void;
  tag(view: TagPageView): void;
  sentences(view: SentencesView): void;
  /** Draws the page of search results, the shell's search box under its `h1`, until another page is drawn. */
  search(view: SearchPageView): void;
  notice(view: NoticeView): void;
  /** Puts first in the page just drawn the note of a site built from a list of feature files, which reads `words`. */
  onlyNote(words: string): void;
}
