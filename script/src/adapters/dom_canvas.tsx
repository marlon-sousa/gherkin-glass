// Adapter: the canvas port over a shell's markup, its `main`, its breadcrumb and its live region, with the front page its `main` may arrive holding.

import { drawn } from "./jsx/drawing.ts";
import type { Child } from "./jsx/jsx_runtime.ts";
import { MessageParts } from "./jsx/message_parts.tsx";
import type { PageFrame } from "../policies/page_frame.ts";
import type { Canvas } from "../ports/canvas.ts";

export class DomCanvas implements Canvas {
  private readonly breadcrumb: HTMLElement | null;
  private readonly liveRegion: HTMLElement | null;
  /** What the `main` arrived holding, and the title the shell arrived with, when it holds the front page. */
  private readonly frontPage: { readonly nodes: readonly Node[]; readonly title: string } | null;
  /** The shell's search box, and the place in the shell it is put back in when the `main` holding it is drawn again. */
  private readonly searchBox: { readonly form: HTMLElement; readonly parent: Node; readonly before: Node | null } | null;

  private constructor(
    private readonly document: Document,
    readonly main: HTMLElement,
  ) {
    this.breadcrumb = this.document.querySelector<HTMLElement>("nav.ghg-breadcrumb");
    this.liveRegion = this.document.getElementById("ghg_announcement");
    // Copies, so that what a visit adds to the front page, such as its buttons' listeners, is not there the next time.
    this.frontPage = main.childElementCount === 0 ? null : { nodes: [...main.childNodes].map((node) => node.cloneNode(true)), title: this.document.title };
    const form = this.document.querySelector<HTMLElement>("form.ghg-search");
    this.searchBox = form?.parentNode ? { form, parent: form.parentNode, before: form.nextSibling } : null;
  }

  /** Null for a document that is not a shell, with no `main` to draw in. */
  static of(document: Document): DomCanvas | null {
    const main = document.getElementById("ghg_main");
    return main === null ? null : new DomCanvas(document, main);
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
    const last = frame.crumbs.length - 1;
    this.breadcrumb?.replaceChildren(
      ...drawn(
        this.document,
        <ol>
          {frame.crumbs.map((crumb, place) => (
            <li>
              <a href={crumb.address} aria-current={place === last && "page"}>
                <MessageParts parts={crumb.words} lang={crumb.lang} />
              </a>
            </li>
          ))}
        </ol>,
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
      takeFocus(heading);
      this.document.defaultView?.scrollTo(0, 0);
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

  holdsFrontPage(): boolean {
    return this.frontPage !== null;
  }

  restoreFrontPage(): string | null {
    if (this.frontPage === null) {
      return null;
    }
    this.draw(this.frontPage.nodes.map((node) => node.cloneNode(true)));
    return this.frontPage.title;
  }

  drawFailure(words: string): void {
    this.draw(<p>{words}</p>);
  }

  /** The shell's search box, to draw in the `main` of the page of search results. */
  searchForm(): HTMLElement | null {
    return this.searchBox?.form ?? null;
  }

  /** Puts what `page` draws in the `main` in place of what it held, the search box back in its place in the shell unless `page` holds it. */
  draw(page: Child): void {
    const nodes = drawn(this.document, page);
    const box = this.searchBox;
    if (box !== null && !nodes.includes(box.form)) {
      box.parent.insertBefore(box.form, box.before);
    }
    this.main.replaceChildren(...nodes);
  }
}

/** An element that is not a link, a button or a field takes the focus from a script only with `tabindex`. */
function takeFocus(element: HTMLElement): void {
  if (!element.hasAttribute("tabindex") && !element.matches("a[href], button, input, select, textarea, summary")) {
    element.setAttribute("tabindex", "-1");
  }
  element.focus({ preventScroll: true });
}

function focusAndShow(element: HTMLElement): void {
  takeFocus(element);
  element.scrollIntoView({ block: "start" });
}
