// Port: each kind of page, drawn from its view in the shell's `main`.

import type { FeatureView } from "../policies/feature_view.ts";

/** The page of rule 239, or the paragraph of rule 243. */
export interface NoticeView {
  readonly heading: string;
  readonly text: string;
}

export interface Pages {
  feature(view: FeatureView): void;
  notice(view: NoticeView): void;
  /** Puts first in the page just drawn the note of a site built from a list of feature files, which reads `words`. */
  onlyNote(words: string): void;
}
