// Adapter: the pages port, each kind of page drawn by its own drawing in the canvas's `main`.

import type { DomCanvas } from "./dom_canvas.ts";
import { featurePage } from "./dom_feature_page.ts";
import { folderPage } from "./dom_folder_page.ts";
import { otherVersionsOf } from "./dom_other_versions.ts";
import { resultsPage } from "./dom_results_page.ts";
import { sentencesPage } from "./dom_sentences_page.ts";
import { tagIndex, tagPage } from "./dom_tag_pages.ts";
import type { ElementBuilder } from "./element_builder.ts";
import type { FeatureView } from "../policies/feature_view.ts";
import type { FolderView } from "../policies/folder_view.ts";
import type { PageInVersions } from "../policies/other_versions.ts";
import type { SearchPageView } from "../policies/search_page_view.ts";
import type { SentencesView } from "../policies/sentences_view.ts";
import type { TagIndexView, TagPageView } from "../policies/tag_views.ts";
import type { NoticeView, Pages } from "../ports/pages.ts";
import type { Words } from "../ports/words.ts";

export class DomPages implements Pages {
  private readonly make: ElementBuilder;
  private readonly canvas: DomCanvas;
  private readonly words: Words;
  /** The shell's search box, and the place in the shell it is put back in. */
  private readonly searchBox: { readonly form: HTMLElement; readonly parent: Node; readonly before: Node | null } | null;

  constructor(make: ElementBuilder, canvas: DomCanvas, words: Words) {
    this.make = make;
    this.canvas = canvas;
    this.words = words;
    const form = make.document.querySelector<HTMLElement>("form.ghg-search");
    this.searchBox = form?.parentNode ? { form, parent: form.parentNode, before: form.nextSibling } : null;
  }

  feature(view: FeatureView, versions: PageInVersions | null): void {
    const inVersions = (anchor: string | null): Node | null => {
      const lists = anchor === null ? undefined : versions?.blocks.get(anchor);
      return lists === undefined ? null : otherVersionsOf(this.make, lists);
    };
    const pageLists = versions?.page ?? null;
    this.draw(featurePage(this.make, view, this.words.text("page-contents"), (block) => inVersions(block.anchor), pageLists && otherVersionsOf(this.make, pageLists)));
  }

  folder(view: FolderView): void {
    this.draw(folderPage(this.make, view));
  }

  tagIndex(view: TagIndexView): void {
    this.draw(tagIndex(this.make, view));
  }

  tag(view: TagPageView): void {
    this.draw(tagPage(this.make, view));
  }

  sentences(view: SentencesView): void {
    this.draw(sentencesPage(this.make, view));
  }

  search(view: SearchPageView): void {
    this.draw(resultsPage(this.make, view, this.searchBox?.form ?? null));
  }

  notice(view: NoticeView): void {
    this.draw([this.make.make("h1", {}, view.heading), this.make.make("p", {}, view.text)]);
  }

  onlyNote(words: string): void {
    this.canvas.main.prepend(this.make.make("p", { class: "ghg-only-note", role: "note" }, words));
  }

  private draw(nodes: readonly Node[]): void {
    const box = this.searchBox;
    // Every page but the page of search results has the search box in its place in the shell.
    if (box !== null && !nodes.includes(box.form)) {
      box.parent.insertBefore(box.form, box.before);
    }
    this.canvas.draw(nodes);
  }
}
