// Adapter: the pages port, each kind of page drawn by its own markup in the canvas's `main`.

import type { DomCanvas } from "./dom_canvas.tsx";
import { featurePage } from "./dom_feature_page.tsx";
import { folderPage } from "./dom_folder_page.tsx";
import { openOtherVersions, otherVersionsOf } from "./dom_other_versions.tsx";
import { resultsPage } from "./dom_results_page.tsx";
import { sentencesPage } from "./dom_sentences_page.tsx";
import { tagIndex, tagPage } from "./dom_tag_pages.tsx";
import { drawn } from "./jsx/drawing.ts";
import type { Markup } from "./jsx/jsx_runtime.ts";
import type { FeatureView } from "../policies/feature_view.ts";
import type { FolderView } from "../policies/folder_view.ts";
import type { PageInVersions } from "../policies/other_versions.ts";
import type { SearchPageView } from "../policies/search_page_view.ts";
import type { SentencesView } from "../policies/sentences_view.ts";
import type { TagIndexView, TagPageView } from "../policies/tag_views.ts";
import type { NoticeView, Pages } from "../ports/pages.ts";
import type { Words } from "../ports/words.ts";

export class DomPages implements Pages {
  constructor(
    private readonly document: Document,
    private readonly canvas: DomCanvas,
    private readonly words: Words,
  ) {}

  feature(view: FeatureView, versions: PageInVersions | null): void {
    const inVersions = (anchor: string | null): Markup | null => {
      const lists = anchor === null ? undefined : versions?.blocks.get(anchor);
      return lists === undefined ? null : otherVersionsOf(lists);
    };
    const pageLists = versions?.page ?? null;
    this.canvas.draw(featurePage(view, this.words.text("page-contents"), (block) => inVersions(block.anchor), pageLists && otherVersionsOf(pageLists)));
    openOtherVersions(this.canvas.main);
  }

  folder(view: FolderView): void {
    this.canvas.draw(folderPage(view));
  }

  tagIndex(view: TagIndexView): void {
    this.canvas.draw(tagIndex(view));
  }

  tag(view: TagPageView): void {
    this.canvas.draw(tagPage(view));
  }

  sentences(view: SentencesView): void {
    this.canvas.draw(sentencesPage(view));
  }

  search(view: SearchPageView): void {
    this.canvas.draw(resultsPage(view, this.canvas.searchForm()));
  }

  notice(view: NoticeView): void {
    this.canvas.draw(
      <>
        <h1>{view.heading}</h1>
        <p>{view.text}</p>
      </>,
    );
  }

  onlyNote(words: string): void {
    this.canvas.main.prepend(...drawn(this.document, <p class="ghg-only-note" role="note">{words}</p>));
  }
}
