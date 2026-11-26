// Service: the page an address names, drawn in the shell, the focus kept in the `main` while it is drawn and moved, with the title said, once a page reached from another is drawn, as docs/BEHAVIOUR.md, rule 240, describes.

import { type PageName, type Target, readTarget, targetAddress } from "../policies/address.ts";
import type { PageFrame } from "../policies/page_frame.ts";
import type { Address, Canvas } from "../ports/canvas.ts";
import type { Timer } from "../ports/timer.ts";

/** The wait between a page reached from another being drawn and the focus moving, measured with one screen reader and one browser, as design note 0173 records. */
export const SETTLE_MILLISECONDS = 150;

/** Fetches what the page `page` names needs, and resolves with what draws it in the canvas's `main` and returns its frame; rejects when the site's data it needs cannot be fetched or read. */
export type PageDrawing = (page: PageName) => Promise<() => PageFrame>;

/** Starts the behaviour of the page just drawn, such as its blocks' buttons, and returns what expands every block an element is inside. */
export type PageBehaviour = (page: PageName) => (id: string) => void;

export interface NavigationParts {
  readonly canvas: Canvas;
  readonly address: Address;
  readonly timer: Timer;
  readonly draw: PageDrawing;
  readonly behave: PageBehaviour;
  /** The words of the paragraph drawn when a page cannot be loaded. */
  readonly failure: string;
  /** Called when a page cannot be loaded, to disable what cannot act without the site's data. */
  readonly onFailure: () => void;
}

export async function startNavigation(parts: NavigationParts): Promise<void> {
  const { canvas, address, timer } = parts;
  let shown: string | null = null;
  let reveal: (id: string) => void = () => {};
  let drawing = 0;
  let hash = address.hash();

  /** Moves the focus to `id`, or to the page's `h1` when the page has no such element and the reader came from another page. */
  const focus = (id: string | null, toHeading: boolean): void => {
    if (id !== null) {
      reveal(id);
      if (canvas.focusElement(id)) {
        return;
      }
    }
    if (toHeading) {
      canvas.focusHeading();
    }
  };

  const draw = async (page: PageName, anchor: string | null, reached: boolean): Promise<void> => {
    drawing += 1;
    const mine = drawing;
    timer.cancel();
    canvas.focusMain();
    canvas.setBusy(true);
    let drawn: () => PageFrame;
    try {
      drawn = await parts.draw(page);
    } catch {
      if (mine === drawing) {
        shown = null;
        canvas.drawFailure(parts.failure);
        canvas.setBusy(false);
        parts.onFailure();
      }
      return;
    }
    if (mine !== drawing) {
      return;
    }
    const frame = drawn();
    shown = targetAddress(page);
    canvas.setFrame(frame);
    reveal = parts.behave(page);
    canvas.setBusy(false);
    if (!reached) {
      canvas.setTitle(frame.title);
      focus(anchor, false);
      return;
    }
    timer.after(SETTLE_MILLISECONDS, () => {
      canvas.setTitle(frame.title);
      canvas.announce(frame.title);
      focus(anchor, true);
    });
  };

  const show = async (target: Target, reached: boolean): Promise<void> => {
    if (target.kind === "element") {
      if (shown === null) {
        await draw({ kind: "home" }, null, false);
      }
      focus(target.id, false);
      return;
    }
    if (shown === targetAddress(target.page)) {
      timer.cancel();
      focus(target.anchor, true);
      return;
    }
    await draw(target.page, target.anchor, reached);
  };

  address.onChange((changed) => {
    const target = readTarget(changed);
    if (target.kind === "element") {
      // An element of the page shown leaves the address naming the page, so that a reload draws it again.
      address.replace(hash);
    } else {
      hash = changed;
    }
    void show(target, true);
  });
  await show(readTarget(hash), false);
}
