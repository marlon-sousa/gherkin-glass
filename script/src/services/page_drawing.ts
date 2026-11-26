// Service: the page an address names, its view worked out from the descriptor and the objects it needs, drawn by the pages port, with the frame the shell shows around it.

import type { PageName } from "../policies/address.ts";
import { featureView } from "../policies/feature_view.ts";
import { folderCrumbs, frameOf, homeCrumb, missingFrame, type PageFrame } from "../policies/page_frame.ts";
import type { Canvas } from "../ports/canvas.ts";
import type { Pages } from "../ports/pages.ts";
import type { Descriptor, Store } from "../ports/store.ts";
import type { Words } from "../ports/words.ts";
import type { PageDrawing } from "./navigation.ts";

export interface DrawingParts {
  readonly store: Store;
  readonly words: Words;
  readonly pages: Pages;
  readonly canvas: Canvas;
}

export function pageDrawing(parts: DrawingParts): PageDrawing {
  return async (page) => {
    const descriptor = await parts.store.descriptor();
    if (page.kind === "home") {
      // The front page arrives in its shell with the note of rule 207 already.
      return home(parts, descriptor);
    }
    const draw = await drawingOf(parts, descriptor, page);
    return () => {
      const frame = draw();
      if (descriptor.only) {
        parts.pages.onlyNote(parts.words.text("only-note"));
      }
      return frame;
    };
  };
}

function drawingOf(parts: DrawingParts, descriptor: Descriptor, page: PageName): Promise<() => PageFrame> | (() => PageFrame) {
  switch (page.kind) {
    case "feature":
      return feature(parts, descriptor, page.page);
    default:
      return missing(parts, descriptor);
  }
}

function home(parts: DrawingParts, descriptor: Descriptor): () => PageFrame {
  return () => {
    const title = parts.canvas.restoreFrontPage();
    return title === null ? missing(parts, descriptor)() : { title, lang: "en", crumbs: [homeCrumb(descriptor.title)] };
  };
}

async function feature(parts: DrawingParts, descriptor: Descriptor, page: string): Promise<() => PageFrame> {
  const entry = descriptor.features.find((candidate) => candidate.page === page);
  if (entry === undefined) {
    return missing(parts, descriptor);
  }
  const view = featureView(parts.words, descriptor, entry, await parts.store.object(entry.object));
  return () => {
    parts.pages.feature(view);
    return frameOf(descriptor.title, view.title, view.lang, folderCrumbs(entry.page), { kind: "feature", page }, view.heading);
  };
}

function missing(parts: DrawingParts, descriptor: Descriptor): () => PageFrame {
  return () => {
    parts.pages.notice({ heading: parts.words.text("page-missing"), text: parts.words.text("page-missing-text") });
    return missingFrame(parts.words, descriptor.title);
  };
}
