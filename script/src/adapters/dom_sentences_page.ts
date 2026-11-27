// Adapter: the page of step sentences drawn from its view, in the elements sentences_page.html, sentences_group.html, sentence.html and sentence_use.html in src/policies/rendering write.

import type { ElementBuilder } from "./element_builder.ts";
import type { Piece } from "../policies/heading_words.ts";
import type { SentenceUseView, SentenceView, SentencesView } from "../policies/sentences_view.ts";

export function sentencesPage(make: ElementBuilder, view: SentencesView): Node[] {
  return make.nodes([
    make.make("h1", {}, view.heading),
    view.none && make.make("p", {}, view.none),
    ...view.groups.flatMap((group) => [make.make("h2", {}, group.heading), ...group.sentences.flatMap((sentence) => sentenceOf(make, sentence))]),
  ]);
}

function sentenceOf(make: ElementBuilder, sentence: SentenceView): Node[] {
  return [make.make("h3", { lang: sentence.lang }, sentence.text), make.make("p", {}, sentence.count), make.make("ul", {}, ...sentence.uses.map((use) => useOf(make, use)))];
}

function useOf(make: ElementBuilder, use: SentenceUseView): HTMLLIElement {
  const link = make.make(
    "a",
    { href: use.address },
    ...drawn(make, use.link, (slot) => (slot === "heading" ? [make.make("bdi", {}, ...make.words(use.heading, use.lang))] : [make.make("bdi", {}, use.path)])),
  );
  if (use.item === null) {
    return make.make("li", {}, link);
  }
  return make.make(
    "li",
    {},
    ...drawn(make, use.item, (slot) => (slot === "link" ? [link] : use.values.flatMap((value, place) => [...(place === 0 ? [] : [use.separator]), make.make("bdi", { lang: use.lang }, value)]))),
  );
}

/** The words of `pieces`, each slot drawn by `slotted`. */
function drawn(make: ElementBuilder, pieces: readonly Piece[], slotted: (slot: string) => readonly (Node | string)[]): Node[] {
  return make.nodes(pieces.flatMap((piece) => ("slot" in piece ? slotted(piece.slot) : [piece.words])));
}
