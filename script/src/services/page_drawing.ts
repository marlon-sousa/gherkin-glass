// Service: the page an address names, its view worked out from the descriptor and the objects it needs, drawn by the pages port, with the frame the shell shows around it.

import { type PageName, targetAddress } from "../policies/address.ts";
import { featureView } from "../policies/feature_view.ts";
import { folderView } from "../policies/folder_view.ts";
import { searchPageView } from "../policies/search_page_view.ts";
import { sentencesView } from "../policies/sentences_view.ts";
import { tagIndexView, tagPageView } from "../policies/tag_views.ts";
import { plain } from "../policies/heading_words.ts";
import { type Crumb, folderCrumbs, frameOf, homeCrumb, missingFrame, type PageFrame } from "../policies/page_frame.ts";
import type { Canvas } from "../ports/canvas.ts";
import type { Pages } from "../ports/pages.ts";
import type { Descriptor, Store } from "../ports/store.ts";
import type { Words } from "../ports/words.ts";
import { loadedFeatures } from "./loaded_features.ts";
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
    const draw = await drawingOf(parts, descriptor, page);
    return () => {
      const frame = draw();
      if (frame === null) {
        // The front page arrives in its shell with the note of rule 207 already.
        return home(parts, descriptor);
      }
      if (descriptor.only) {
        parts.pages.onlyNote(parts.words.text("only-note"));
      }
      return frame;
    };
  };
}

/** Null when the page is the front page of a site or a preview, which its shell holds. */
type Drawing = () => PageFrame | null;

function drawingOf(parts: DrawingParts, descriptor: Descriptor, page: PageName): Promise<Drawing> | Drawing {
  switch (page.kind) {
    case "home":
      return parts.canvas.holdsFrontPage() ? () => null : folder(parts, descriptor, "");
    case "folder":
      return folder(parts, descriptor, page.folder);
    case "feature":
      return feature(parts, descriptor, page.page);
    case "tags":
      return tagIndex(parts, descriptor);
    case "tag":
      return tag(parts, descriptor, page.tag);
    case "steps":
      return sentences(parts, descriptor);
    case "search":
      return search(parts, descriptor, page.words);
  }
}

function home(parts: DrawingParts, descriptor: Descriptor): PageFrame {
  const title = parts.canvas.restoreFrontPage();
  return title === null ? missing(parts, descriptor)() : { title, lang: "en", crumbs: [homeCrumb(descriptor.title)] };
}

async function folder(parts: DrawingParts, descriptor: Descriptor, name: string): Promise<() => PageFrame> {
  const view = folderView(parts.words, descriptor, await loadedFeatures(parts.store, descriptor), name);
  if (view === null) {
    return missing(parts, descriptor);
  }
  return () => {
    parts.pages.folder(view);
    const crumbs = name === "" ? [homeCrumb(descriptor.title)] : frameOf(descriptor.title, view.title, "en", folderCrumbs(name.slice(0, -1)), { kind: "folder", folder: name }, plain(view.heading)).crumbs;
    return { title: view.title, lang: "en", crumbs };
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

/** The crumb of the index of tags, under the home page's. */
function tagIndexCrumb(parts: DrawingParts): Crumb {
  return { address: targetAddress({ kind: "tags" }), words: plain(parts.words.text("tag-index-title")), lang: null };
}

async function tagIndex(parts: DrawingParts, descriptor: Descriptor): Promise<() => PageFrame> {
  const view = tagIndexView(parts.words, descriptor, await loadedFeatures(parts.store, descriptor));
  return () => {
    parts.pages.tagIndex(view);
    return { title: view.title, lang: "en", crumbs: [homeCrumb(descriptor.title), tagIndexCrumb(parts)] };
  };
}

async function tag(parts: DrawingParts, descriptor: Descriptor, name: string): Promise<() => PageFrame> {
  const view = tagPageView(parts.words, descriptor, await loadedFeatures(parts.store, descriptor), name);
  if (view === null) {
    return missing(parts, descriptor);
  }
  return () => {
    parts.pages.tag(view);
    return frameOf(descriptor.title, view.title, "en", [tagIndexCrumb(parts)], { kind: "tag", tag: name }, view.heading);
  };
}

async function sentences(parts: DrawingParts, descriptor: Descriptor): Promise<() => PageFrame> {
  const view = sentencesView(parts.words, descriptor, await loadedFeatures(parts.store, descriptor));
  return () => {
    parts.pages.sentences(view);
    return frameOf(descriptor.title, view.title, "en", [], { kind: "steps" }, plain(view.heading));
  };
}

function search(parts: DrawingParts, descriptor: Descriptor, words: string): () => PageFrame {
  const view = searchPageView(parts.words, descriptor.title, words);
  return () => {
    parts.pages.search(view);
    // The breadcrumb's own link reads the same whatever the words searched for.
    return frameOf(descriptor.title, view.title, "en", [], { kind: "search", words: "" }, plain(view.heading));
  };
}

function missing(parts: DrawingParts, descriptor: Descriptor): () => PageFrame {
  return () => {
    parts.pages.notice({ heading: parts.words.text("page-missing"), text: parts.words.text("page-missing-text") });
    return missingFrame(parts.words, descriptor.title);
  };
}
