// Adapter: the pages port, each kind of page drawn by its own drawing in the canvas's `main`.

import type { DomCanvas } from "./dom_canvas.ts";
import { featurePage } from "./dom_feature_page.ts";
import type { ElementBuilder } from "./element_builder.ts";
import type { FeatureView } from "../policies/feature_view.ts";
import type { NoticeView, Pages } from "../ports/pages.ts";
import type { Words } from "../ports/words.ts";

export class DomPages implements Pages {
  private readonly make: ElementBuilder;
  private readonly canvas: DomCanvas;
  private readonly words: Words;

  constructor(make: ElementBuilder, canvas: DomCanvas, words: Words) {
    this.make = make;
    this.canvas = canvas;
    this.words = words;
  }

  feature(view: FeatureView): void {
    this.draw(featurePage(this.make, view, this.words.text("page-contents"), () => null, null));
  }

  notice(view: NoticeView): void {
    this.draw([this.make.make("h1", {}, view.heading), this.make.make("p", {}, view.text)]);
  }

  onlyNote(words: string): void {
    this.canvas.main.prepend(this.make.make("p", { class: "ghg-only-note", role: "note" }, words));
  }

  private draw(nodes: readonly Node[]): void {
    this.canvas.draw(nodes);
  }
}
