// Adapter: the markup of the page of step sentences, drawn from its view, in the elements the stylesheet styles.

import type { Child, Markup } from "./jsx/jsx_runtime.ts";
import { Words } from "./jsx/words.tsx";
import type { Piece } from "../policies/heading_words.ts";
import type { SentenceUseView, SentenceView, SentencesView } from "../policies/sentences_view.ts";

export function sentencesPage(view: SentencesView): Markup {
  return (
    <>
      <h1>{view.heading}</h1>
      {view.none && <p>{view.none}</p>}
      {view.groups.map((group) => [<h2>{group.heading}</h2>, group.sentences.map(sentenceOf)])}
    </>
  );
}

function sentenceOf(sentence: SentenceView): Markup {
  return (
    <>
      <h3 lang={sentence.lang}>{sentence.text}</h3>
      <p>{sentence.count}</p>
      <ul>{sentence.uses.map(useOf)}</ul>
    </>
  );
}

function useOf(use: SentenceUseView): Markup {
  const link = (
    <a href={use.address}>
      {slotted(use.link, (slot) =>
        slot === "heading" ? (
          <bdi>
            <Words parts={use.heading} lang={use.lang} />
          </bdi>
        ) : (
          <bdi>{use.path}</bdi>
        ),
      )}
    </a>
  );
  if (use.item === null) {
    return <li>{link}</li>;
  }
  return <li>{slotted(use.item, (slot) => (slot === "link" ? link : use.values.map((value, place) => [place > 0 && use.separator, <bdi lang={use.lang}>{value}</bdi>])))}</li>;
}

/** The words of `pieces`, each slot drawn by `slot`. */
function slotted(pieces: readonly Piece[], slot: (name: string) => Child): Child {
  return pieces.map((piece) => ("slot" in piece ? slot(piece.slot) : piece.words));
}
