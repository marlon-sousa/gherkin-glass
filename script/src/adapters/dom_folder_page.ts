// Adapter: a folder's page drawn from its view, in the elements and classes folder_page.html, subfolder_link.html and folder_feature.html in src/policies/rendering write.

import { contentsList } from "./dom_feature_page.ts";
import type { ElementBuilder } from "./element_builder.ts";
import type { FolderFeatureView, FolderView, LinkView } from "../policies/folder_view.ts";

/** The content of a folder's `main`. */
export function folderPage(make: ElementBuilder, view: FolderView): Node[] {
  return make.nodes([
    make.make("h1", {}, view.heading),
    view.totals.length > 0 && make.make("ul", { class: "ghg-totals" }, ...view.totals.map((total) => make.make("li", {}, total))),
    view.tagIndex && make.make("p", { class: "ghg-tag-index-link" }, link(make, view.tagIndex)),
    view.sentences && make.make("p", { class: "ghg-sentences" }, link(make, view.sentences)),
    view.nothingGiven && make.make("p", { class: "ghg-nothing-given" }, view.nothingGiven),
    view.folders && make.make("h2", {}, view.folders.heading),
    view.folders && make.make("ul", { class: "ghg-folders" }, ...view.folders.links.map((folder) => make.make("li", {}, link(make, folder)))),
    view.features && make.make("h2", {}, view.features.heading),
    view.features && make.make("ul", { class: "ghg-features" }, ...view.features.entries.map((feature) => featureEntry(make, feature))),
  ]);
}

function link(make: ElementBuilder, view: LinkView): HTMLAnchorElement {
  return make.make("a", { href: view.address }, view.text);
}

function featureEntry(make: ElementBuilder, feature: FolderFeatureView): HTMLLIElement {
  const heading = make.make("a", { href: feature.address }, ...make.words(feature.heading, feature.lang));
  if (feature.countName === null) {
    return make.make("li", { class: "ghg-feature" }, heading, "\n", make.make("span", { class: "ghg-count" }, feature.count));
  }
  return make.make(
    "li",
    { class: "ghg-feature" },
    heading,
    "\n",
    make.make("button", { type: "button", class: "ghg-toggle", "aria-expanded": "false", "aria-label": feature.countName }, feature.count),
    make.make("div", { class: "ghg-content", hidden: true }, contentsList(make, feature.contents, feature.lang)),
  );
}
