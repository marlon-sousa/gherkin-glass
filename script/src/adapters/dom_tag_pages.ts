// Adapter: the index of tags and a tag's page drawn from their views, in the elements and classes tag_index.html, tag_index_entry.html, tag_page.html, tag_feature.html, tag_scenario.html and outside_link.html in src/policies/rendering write.

import type { ElementBuilder } from "./element_builder.ts";
import type { LinkInWords, TagFeatureView, TagIndexView, TagPageView } from "../policies/tag_views.ts";

export function tagIndex(make: ElementBuilder, view: TagIndexView): Node[] {
  return make.nodes([
    make.make("h1", {}, view.title),
    view.entries === null
      ? make.make("p", {}, view.empty)
      : make.make("ul", { class: "ghg-tag-index" }, ...view.entries.map((entry) => make.make("li", {}, ...linkInWords(make, entry, make.words(entry.words))))),
  ]);
}

export function tagPage(make: ElementBuilder, view: TagPageView): Node[] {
  return make.nodes([
    make.make("h1", {}, ...make.words(view.heading)),
    view.outside && make.make("p", {}, make.make("a", { class: "ghg-tag-link", href: view.outside.address }, view.outside.text)),
    ...view.features.flatMap((feature) => tagFeature(make, feature)),
  ]);
}

function tagFeature(make: ElementBuilder, feature: TagFeatureView): Node[] {
  return [
    make.make("h2", {}, make.make("a", { href: feature.address }, ...make.words(feature.before, feature.lang), make.make("span", { class: "ghg-tag-path" }, feature.path), ...make.words(feature.after))),
    feature.scenarios === null
      ? make.make("p", {}, feature.none)
      : make.make("ul", { class: "ghg-tag-scenarios" }, ...feature.scenarios.map((scenario) => make.make("li", {}, ...linkInWords(make, scenario, [make.make("bdi", {}, ...make.words(scenario.words, feature.lang))])))),
  ];
}

/** The words before the link, the link holding `inner`, and the words after it. */
function linkInWords(make: ElementBuilder, link: LinkInWords, inner: readonly Node[]): Node[] {
  return [...make.words(link.before), make.make("a", { href: link.address }, ...inner), ...make.words(link.after)];
}
