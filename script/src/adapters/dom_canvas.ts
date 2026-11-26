// Adapter: the canvas port over a shell's markup, its `main`, its breadcrumb and its live region, with the front page its `main` may arrive holding.

import type { ElementBuilder } from "./element_builder.ts";
import type { PageFrame } from "../policies/page_frame.ts";
import type { Canvas } from "../ports/canvas.ts";

export class DomCanvas implements Canvas {
  private readonly make: ElementBuilder;
  private readonly document: Document;
  readonly main: HTMLElement;
  private readonly breadcrumb: HTMLElement | null;
  private readonly liveRegion: HTMLElement | null;
  /** What the `main` arrived holding, and the title the shell arrived with, when it holds the front page. */
  private readonly frontPage: { readonly nodes: readonly Node[]; readonly title: string } | null;

  private constructor(make: ElementBuilder, main: HTMLElement) {
    this.make = make;
    this.document = make.document;
    this.main = main;
    this.breadcrumb = this.document.querySelector<HTMLElement>("nav.ghg-breadcrumb");
    this.liveRegion = this.document.getElementById("ghg_announcement");
    this.frontPage = main.childElementCount === 0 ? null : { nodes: [...main.childNodes], title: this.document.title };
  }

  /** Null for a document that is not a shell, with no `main` to draw in. */
  static of(make: ElementBuilder): DomCanvas | null {
    const main = make.document.getElementById("ghg_main");
    return main === null ? null : new DomCanvas(make, main);
  }

  setBusy(busy: boolean): void {
    if (busy) {
      this.main.setAttribute("aria-busy", "true");
    } else {
      this.main.removeAttribute("aria-busy");
    }
  }

  /** The element with the focus may be about to be replaced, which would drop the focus to the top of the document. */
  focusMain(): void {
    this.main.focus({ preventScroll: true });
  }

  setFrame(frame: PageFrame): void {
    this.document.documentElement.lang = frame.lang;
    this.breadcrumb?.replaceChildren(
      this.make.make(
        "ol",
        {},
        ...frame.crumbs.map((crumb, place) =>
          this.make.make("li", {}, this.make.make("a", { href: crumb.address, "aria-current": place === frame.crumbs.length - 1 && "page" }, ...this.make.words(crumb.words, crumb.lang))),
        ),
      ),
    );
  }

  setTitle(title: string): void {
    this.document.title = title;
  }

  announce(words: string): void {
    if (this.liveRegion === null) {
      return;
    }
    // A live region says only what changes in it, so the same words are said again after it is emptied.
    this.liveRegion.textContent = "";
    this.liveRegion.textContent = words;
  }

  focusHeading(): void {
    const heading = this.main.querySelector<HTMLElement>("h1");
    if (heading !== null) {
      focusAndShow(heading);
    }
  }

  focusElement(id: string): boolean {
    const element = this.document.getElementById(id);
    if (element === null) {
      return false;
    }
    focusAndShow(element);
    return true;
  }

  restoreFrontPage(): string | null {
    if (this.frontPage === null) {
      return null;
    }
    this.main.replaceChildren(...this.frontPage.nodes);
    return this.frontPage.title;
  }

  drawFailure(words: string): void {
    this.main.replaceChildren(this.make.make("p", {}, words));
  }

  /** Puts `nodes` in the `main` in place of what it held. */
  draw(nodes: readonly Node[]): void {
    this.main.replaceChildren(...nodes);
  }
}

/** An element that is not a link, a button or a field takes the focus from a script only with `tabindex`. */
function focusAndShow(element: HTMLElement): void {
  if (!element.hasAttribute("tabindex") && !element.matches("a[href], button, input, select, textarea, summary")) {
    element.setAttribute("tabindex", "-1");
  }
  element.focus({ preventScroll: true });
  element.scrollIntoView({ block: "start" });
}
