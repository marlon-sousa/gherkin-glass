// Compiled from script/ by cargo xtask build. Do not edit: change script/src and build again.

// src/adapters/followed_link.ts
function followedLink(event) {
  if (event.defaultPrevented || event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) {
    return null;
  }
  const target = event.target;
  return typeof target?.closest === "function" ? target.closest("a[href]") : null;
}

// src/adapters/dom_address.ts
var DomAddress = class {
  window;
  constructor(window2) {
    this.window = window2;
  }
  hash() {
    return this.window.location.hash;
  }
  replace(hash) {
    const address = new URL(this.window.location.href);
    address.hash = hash;
    this.window.history.replaceState(this.window.history.state, "", address.href);
  }
  onChange(handler) {
    this.window.addEventListener("hashchange", () => handler(this.window.location.hash));
    this.window.document.addEventListener("click", (event) => {
      const hash = followedLink(event)?.getAttribute("href") ?? "";
      if (!hash.startsWith("#")) {
        return;
      }
      if (hash.startsWith("##")) {
        event.preventDefault();
        handler(hash);
      } else if (hash === this.window.location.hash || hash === "#" && this.window.location.hash === "") {
        handler(hash);
      }
    });
  }
};

// src/adapters/dom_canvas.ts
var DomCanvas = class _DomCanvas {
  make;
  document;
  main;
  breadcrumb;
  liveRegion;
  /** What the `main` arrived holding, and the title the shell arrived with, when it holds the front page. */
  frontPage;
  /** The shell's search box, and the place in the shell it is put back in when the `main` holding it is drawn again. */
  searchBox;
  constructor(make, main) {
    this.make = make;
    this.document = make.document;
    this.main = main;
    this.breadcrumb = this.document.querySelector("nav.ghg-breadcrumb");
    this.liveRegion = this.document.getElementById("ghg_announcement");
    this.frontPage = main.childElementCount === 0 ? null : { nodes: [...main.childNodes].map((node) => node.cloneNode(true)), title: this.document.title };
    const form = this.document.querySelector("form.ghg-search");
    this.searchBox = form?.parentNode ? { form, parent: form.parentNode, before: form.nextSibling } : null;
  }
  /** Null for a document that is not a shell, with no `main` to draw in. */
  static of(make) {
    const main = make.document.getElementById("ghg_main");
    return main === null ? null : new _DomCanvas(make, main);
  }
  setBusy(busy) {
    if (busy) {
      this.main.setAttribute("aria-busy", "true");
    } else {
      this.main.removeAttribute("aria-busy");
    }
  }
  /** The element with the focus may be about to be replaced, which would drop the focus to the top of the document. */
  focusMain() {
    this.main.focus({ preventScroll: true });
  }
  setFrame(frame) {
    this.document.documentElement.lang = frame.lang;
    this.breadcrumb?.replaceChildren(
      this.make.make(
        "ol",
        {},
        ...frame.crumbs.map(
          (crumb, place) => this.make.make("li", {}, this.make.make("a", { href: crumb.address, "aria-current": place === frame.crumbs.length - 1 && "page" }, ...this.make.words(crumb.words, crumb.lang)))
        )
      )
    );
  }
  setTitle(title) {
    this.document.title = title;
  }
  announce(words) {
    if (this.liveRegion === null) {
      return;
    }
    this.liveRegion.textContent = "";
    this.liveRegion.textContent = words;
  }
  focusHeading() {
    const heading2 = this.main.querySelector("h1");
    if (heading2 !== null) {
      focusAndShow(heading2);
    }
  }
  focusElement(id) {
    const element = this.document.getElementById(id);
    if (element === null) {
      return false;
    }
    focusAndShow(element);
    return true;
  }
  holdsFrontPage() {
    return this.frontPage !== null;
  }
  restoreFrontPage() {
    if (this.frontPage === null) {
      return null;
    }
    this.draw(this.frontPage.nodes.map((node) => node.cloneNode(true)));
    return this.frontPage.title;
  }
  drawFailure(words) {
    this.draw([this.make.make("p", {}, words)]);
  }
  /** The shell's search box, to draw in the `main` of the page of search results. */
  searchForm() {
    return this.searchBox?.form ?? null;
  }
  /** Puts `nodes` in the `main` in place of what it held, the search box back in its place in the shell unless `nodes` hold it. */
  draw(nodes) {
    const box = this.searchBox;
    if (box !== null && !nodes.includes(box.form)) {
      box.parent.insertBefore(box.form, box.before);
    }
    this.main.replaceChildren(...nodes);
  }
};
function focusAndShow(element) {
  if (!element.hasAttribute("tabindex") && !element.matches("a[href], button, input, select, textarea, summary")) {
    element.setAttribute("tabindex", "-1");
  }
  element.focus({ preventScroll: true });
  element.scrollIntoView({ block: "start" });
}

// src/adapters/dom_feature_parts.ts
function tagList(make, tags) {
  if (tags.length === 0) {
    return null;
  }
  return make.make(
    "ul",
    { class: "ghg-tags" },
    ...tags.map(
      (tag2) => make.make("li", {}, make.make("a", { href: tag2.address }, tag2.text), tag2.link && " ", tag2.link && make.make("a", { class: "ghg-tag-link", href: tag2.link.address }, tag2.link.text))
    )
  );
}
function descriptionOf(make, lines) {
  if (lines.length === 0) {
    return null;
  }
  const paragraph = make.make("p");
  lines.forEach((line, place) => {
    if (place > 0) {
      paragraph.append(make.make("br"), "\n");
    }
    paragraph.append(line);
  });
  return make.make("div", { class: "ghg-description" }, paragraph);
}
function noteOf(make, note) {
  if (note === null) {
    return null;
  }
  const opener = make.make("button", { type: "button", class: "ghg-note-opener", "aria-haspopup": "dialog", "aria-controls": "ghg_note_dialog" }, ...make.words(note.opener));
  return make.make("p", { class: "ghg-keyword-note", role: "note" }, note.before, opener, note.after);
}
function tableOf(make, table2) {
  const head = make.make("thead", {}, make.make("tr", {}, ...table2.header.map((cell) => make.make("th", { scope: "col" }, cell))));
  const body = table2.rows.length === 0 ? null : make.make("tbody", {}, ...table2.rows.map((row) => make.make("tr", {}, ...row.map((cell) => make.make("td", {}, cell)))));
  return make.make("table", { class: "ghg-table" }, head, body);
}
function docStringOf(make, doc) {
  if (doc.media_type === null) {
    return make.make("pre", { class: "ghg-doc-string" }, doc.content);
  }
  return make.make("figure", { class: "ghg-doc-string" }, make.make("figcaption", {}, doc.media_type), make.make("pre", {}, doc.content));
}
function stepList(make, steps) {
  if (steps.length === 0) {
    return null;
  }
  return make.make("ol", { class: "ghg-steps" }, ...steps.map((step2) => stepItem(make, step2)));
}
function stepItem(make, step2) {
  return make.make(
    "li",
    {},
    make.make("span", { class: "ghg-keyword" }, step2.keyword),
    ...step2.parts.map((part2) => partOf(make, part2)),
    step2.table && "\n",
    step2.table && tableOf(make, step2.table),
    step2.docString && "\n",
    step2.docString && docStringOf(make, step2.docString)
  );
}
function partOf(make, part2) {
  switch (part2.kind) {
    case "text":
      return part2.text;
    case "parameter":
      return make.make("span", { class: "ghg-parameter" }, ...part2.parts.map((inner) => inner.kind === "placeholder" ? partOf(make, inner) : inner.text));
    case "number":
    case "placeholder":
      return make.make("span", { class: `ghg-${part2.kind}` }, part2.text);
  }
}

// src/adapters/dom_feature_page.ts
function featurePage(make, view, contentsName, firstInBlock, afterHeading) {
  const controls = view.controls;
  return make.nodes([
    make.make("h1", {}, ...make.words(view.heading)),
    afterHeading,
    noteOf(make, view.note),
    tagList(make, view.tags),
    descriptionOf(make, view.description),
    view.source && make.make("p", { class: "ghg-source" }, make.make("a", { href: view.source.address }, ...make.words(view.source.words))),
    view.contents.length > 0 && make.make("nav", { class: "ghg-contents", "aria-label": contentsName }, contentsList(make, view.contents, null)),
    controls && make.make(
      "div",
      { class: "ghg-controls" },
      make.make("button", { type: "button", class: "ghg-collapse-all" }, controls.collapseAll),
      make.make("button", { type: "button", class: "ghg-expand-all" }, controls.expandAll),
      controls.backgrounds && make.make(
        "label",
        { class: "ghg-option" },
        make.make("input", { type: "checkbox", class: "ghg-inline-backgrounds", autocomplete: "off", checked: controls.backgrounds.checked, disabled: !controls.backgrounds.enabled }),
        ` ${controls.backgrounds.label}`
      )
    ),
    ...view.blocks.map((block2) => blockOf(make, block2, firstInBlock)),
    view.dialog && make.make(
      "dialog",
      { id: "ghg_note_dialog", class: "ghg-dialog", tabindex: "-1", "aria-labelledby": "ghg_note_dialog_title", "aria-describedby": "ghg_note_dialog_text" },
      make.make("h2", { id: "ghg_note_dialog_title", class: "ghg-dialog-title" }, view.dialog.title),
      make.make("p", { id: "ghg_note_dialog_text" }, view.dialog.text),
      make.make("form", { method: "dialog" }, make.make("button", {}, view.dialog.close))
    )
  ]);
}
function contentsList(make, entries, lang) {
  return make.make(
    "ul",
    {},
    ...entries.map((entry) => make.make("li", {}, make.make("a", { href: entry.address }, ...make.words(entry.heading, lang)), entry.entries.length > 0 && contentsList(make, entry.entries, lang)))
  );
}
function blockOf(make, block2, firstInBlock) {
  const heading2 = make.make(headingTag(block2.level), { id: block2.id }, toggle(make, block2.expanded, make.words(block2.heading)));
  const content = make.make(
    "div",
    { class: "ghg-content", hidden: !block2.expanded },
    firstInBlock(block2),
    noteOf(make, block2.note),
    tagList(make, block2.tags),
    descriptionOf(make, block2.description),
    ...block2.backgrounds.map((background) => blockOf(make, background, firstInBlock)),
    block2.stepsHeading && make.make(headingTag(block2.stepsHeading.level), { class: "ghg-steps-heading" }, block2.stepsHeading.text),
    stepList(make, block2.steps),
    block2.examples && examplesList(make, block2.examples, firstInBlock),
    ...block2.blocks.map((inner) => blockOf(make, inner, firstInBlock))
  );
  const unchecked = block2.expandedWhenUnchecked === null ? null : String(block2.expandedWhenUnchecked);
  return make.make("div", { class: `ghg-block ${block2.className}`, "data-expanded-when-unchecked": unchecked }, heading2, content);
}
function examplesList(make, examples, firstInBlock) {
  return make.fragment(
    examples.runsParagraph && make.make("p", { id: examples.runsParagraph.id, class: "ghg-runs" }, examples.runsParagraph.text),
    make.make("ul", { class: "ghg-examples-list" }, ...examples.items.map((item) => examplesItem(make, item, firstInBlock)))
  );
}
function examplesItem(make, item, firstInBlock) {
  if (item.holdsNothing) {
    return make.make("li", { id: item.id, class: "ghg-examples" }, make.make("button", { type: "button", class: "ghg-toggle", "aria-disabled": "true" }, ...make.words(item.label)));
  }
  const content = make.make(
    "div",
    { class: "ghg-content", hidden: true },
    noteOf(make, item.note),
    tagList(make, item.tags),
    descriptionOf(make, item.description),
    item.table && tableOf(make, item.table),
    item.runs && blockOf(make, item.runs, firstInBlock)
  );
  return make.make("li", { id: item.id, class: "ghg-block ghg-examples" }, toggle(make, false, make.words(item.label)), content);
}
function toggle(make, expanded, words) {
  return make.make("button", { type: "button", class: "ghg-toggle", "aria-expanded": String(expanded) }, ...words);
}
function headingTag(level) {
  switch (level) {
    case 2:
      return "h2";
    case 3:
      return "h3";
    case 4:
      return "h4";
    case 5:
      return "h5";
    default:
      return "h6";
  }
}

// src/adapters/dom_folder_page.ts
function folderPage(make, view) {
  return make.nodes([
    make.make("h1", {}, view.heading),
    view.totals.length > 0 && make.make("ul", { class: "ghg-totals" }, ...view.totals.map((total) => make.make("li", {}, total))),
    view.tagIndex && make.make("p", { class: "ghg-tag-index-link" }, link(make, view.tagIndex)),
    view.sentences && make.make("p", { class: "ghg-sentences" }, link(make, view.sentences)),
    view.nothingGiven && make.make("p", { class: "ghg-nothing-given" }, view.nothingGiven),
    view.folders && make.make("h2", {}, view.folders.heading),
    view.folders && make.make("ul", { class: "ghg-folders" }, ...view.folders.links.map((folder2) => make.make("li", {}, link(make, folder2)))),
    view.features && make.make("h2", {}, view.features.heading),
    view.features && make.make("ul", { class: "ghg-features" }, ...view.features.entries.map((feature2) => featureEntry(make, feature2)))
  ]);
}
function link(make, view) {
  return make.make("a", { href: view.address }, view.text);
}
function featureEntry(make, feature2) {
  const heading2 = make.make("a", { href: feature2.address }, ...make.words(feature2.heading, feature2.lang));
  if (feature2.countName === null) {
    return make.make("li", { class: "ghg-feature" }, heading2, "\n", make.make("span", { class: "ghg-count" }, feature2.count));
  }
  return make.make(
    "li",
    { class: "ghg-feature" },
    heading2,
    "\n",
    make.make("button", { type: "button", class: "ghg-toggle", "aria-expanded": "false", "aria-label": feature2.countName }, feature2.count),
    make.make("div", { class: "ghg-content", hidden: true }, contentsList(make, feature2.contents, feature2.lang))
  );
}

// src/adapters/dom_other_versions.ts
function otherVersionsOf(make, view) {
  return disclosure(
    make,
    "ghg-other-versions",
    view.label,
    make.make("div", {}, ...view.groups.map((group) => disclosure(make, "ghg-version-group", group.label, make.make("ul", { class: "ghg-versions" }, ...group.links.map((link2) => linkItem(make, link2))))))
  );
}
function disclosure(make, className, label2, shown) {
  const button = make.make("button", { type: "button", class: "ghg-toggle", "aria-expanded": "false" }, label2);
  shown.classList.add("ghg-content");
  shown.hidden = true;
  button.addEventListener("click", () => {
    const opened = button.getAttribute("aria-expanded") !== "true";
    button.setAttribute("aria-expanded", String(opened));
    shown.hidden = !opened;
  });
  return make.make("div", { class: className }, button, shown);
}
function linkItem(make, link2) {
  return make.make("li", {}, ...link2.item.map((piece) => "slot" in piece ? make.make("a", { href: link2.address }, link2.text) : piece.words));
}

// src/adapters/dom_results_page.ts
function resultsPage(make, view, searchBox) {
  const words = view.words;
  const headings = words.headings;
  return make.nodes([
    make.make("h1", { class: "ghg-search-heading" }, view.heading),
    searchBox,
    make.make("p", { class: "ghg-results-status", role: "status" }),
    make.make("div", {
      class: "ghg-search-groups",
      "data-heading": words.heading,
      "data-title": words.title,
      "data-loading": words.loading,
      "data-failed": words.failed,
      "data-none": words.none,
      "data-count": words.count,
      "data-within": words.within,
      "data-separator": words.separator,
      "data-matching-steps": words.matchingSteps,
      "data-matching-tags": words.matchingTags,
      "data-matching-cells": words.matchingCells,
      "data-feature": headings.feature,
      "data-unnamed-feature": headings.unnamedFeature,
      "data-rule": headings.rule,
      "data-unnamed-rule": headings.unnamedRule,
      "data-background": headings.background,
      "data-unnamed-background": headings.unnamedBackground,
      "data-scenario": headings.scenario,
      "data-unnamed-scenario": headings.unnamedScenario,
      "data-scenario-outline": headings.scenarioOutline,
      "data-unnamed-scenario-outline": headings.unnamedScenarioOutline,
      "data-examples": headings.examples,
      "data-numbered-examples": headings.numberedExamples
    })
  ]);
}

// src/adapters/dom_sentences_page.ts
function sentencesPage(make, view) {
  return make.nodes([
    make.make("h1", {}, view.heading),
    view.none && make.make("p", {}, view.none),
    ...view.groups.flatMap((group) => [make.make("h2", {}, group.heading), ...group.sentences.flatMap((sentence) => sentenceOf(make, sentence))])
  ]);
}
function sentenceOf(make, sentence) {
  return [make.make("h3", { lang: sentence.lang }, sentence.text), make.make("p", {}, sentence.count), make.make("ul", {}, ...sentence.uses.map((use) => useOf(make, use)))];
}
function useOf(make, use) {
  const link2 = make.make(
    "a",
    { href: use.address },
    ...drawn(make, use.link, (slot) => slot === "heading" ? [make.make("bdi", {}, ...make.words(use.heading, use.lang))] : [make.make("bdi", {}, use.path)])
  );
  if (use.item === null) {
    return make.make("li", {}, link2);
  }
  return make.make(
    "li",
    {},
    ...drawn(make, use.item, (slot) => slot === "link" ? [link2] : use.values.flatMap((value, place) => [...place === 0 ? [] : [use.separator], make.make("bdi", { lang: use.lang }, value)]))
  );
}
function drawn(make, pieces2, slotted) {
  return make.nodes(pieces2.flatMap((piece) => "slot" in piece ? slotted(piece.slot) : [piece.words]));
}

// src/adapters/dom_tag_pages.ts
function tagIndex(make, view) {
  return make.nodes([
    make.make("h1", {}, view.title),
    view.entries === null ? make.make("p", {}, view.empty) : make.make("ul", { class: "ghg-tag-index" }, ...view.entries.map((entry) => make.make("li", {}, ...linkInWords(make, entry, make.words(entry.words)))))
  ]);
}
function tagPage(make, view) {
  return make.nodes([
    make.make("h1", {}, ...make.words(view.heading)),
    view.outside && make.make("p", {}, make.make("a", { class: "ghg-tag-link", href: view.outside.address }, view.outside.text)),
    ...view.features.flatMap((feature2) => tagFeature(make, feature2))
  ]);
}
function tagFeature(make, feature2) {
  return [
    make.make("h2", {}, make.make("a", { href: feature2.address }, ...make.words(feature2.before, feature2.lang), make.make("span", { class: "ghg-tag-path" }, feature2.path), ...make.words(feature2.after))),
    feature2.scenarios === null ? make.make("p", {}, feature2.none) : make.make("ul", { class: "ghg-tag-scenarios" }, ...feature2.scenarios.map((scenario) => make.make("li", {}, ...linkInWords(make, scenario, [make.make("bdi", {}, ...make.words(scenario.words, feature2.lang))]))))
  ];
}
function linkInWords(make, link2, inner) {
  return [...make.words(link2.before), make.make("a", { href: link2.address }, ...inner), ...make.words(link2.after)];
}

// src/adapters/dom_pages.ts
var DomPages = class {
  make;
  canvas;
  words;
  constructor(make, canvas, words) {
    this.make = make;
    this.canvas = canvas;
    this.words = words;
  }
  feature(view, versions) {
    const inVersions2 = (anchor) => {
      const lists = anchor === null ? void 0 : versions?.blocks.get(anchor);
      return lists === void 0 ? null : otherVersionsOf(this.make, lists);
    };
    const pageLists = versions?.page ?? null;
    this.draw(featurePage(this.make, view, this.words.text("page-contents"), (block2) => inVersions2(block2.anchor), pageLists && otherVersionsOf(this.make, pageLists)));
  }
  folder(view) {
    this.draw(folderPage(this.make, view));
  }
  tagIndex(view) {
    this.draw(tagIndex(this.make, view));
  }
  tag(view) {
    this.draw(tagPage(this.make, view));
  }
  sentences(view) {
    this.draw(sentencesPage(this.make, view));
  }
  search(view) {
    this.draw(resultsPage(this.make, view, this.canvas.searchForm()));
  }
  notice(view) {
    this.draw([this.make.make("h1", {}, view.heading), this.make.make("p", {}, view.text)]);
  }
  onlyNote(words) {
    this.canvas.main.prepend(this.make.make("p", { class: "ghg-only-note", role: "note" }, words));
  }
  draw(nodes) {
    this.canvas.draw(nodes);
  }
};

// src/adapters/element_builder.ts
var ElementBuilder = class {
  document;
  constructor(document2) {
    this.document = document2;
  }
  make(tag2, attributes = {}, ...children) {
    const made = this.document.createElement(tag2);
    for (const [name2, value] of Object.entries(attributes)) {
      if (value === true) {
        made.setAttribute(name2, "");
      } else if (typeof value === "string") {
        made.setAttribute(name2, value);
      }
    }
    made.append(...this.nodes(children));
    return made;
  }
  /** `children` as nodes, text as text nodes, with nothing for `null` and `false`. */
  nodes(children) {
    return children.flatMap((child) => {
      if (child === null || child === false) {
        return [];
      }
      return [typeof child === "string" ? this.document.createTextNode(child) : child];
    });
  }
  /** `parts`, each isolated one in a `bdi` of its own, with `lang` when it is given. */
  words(parts, lang = null) {
    return parts.map((part2) => part2.isolated ? this.make("bdi", { lang }, part2.text) : this.document.createTextNode(part2.text));
  }
  /** Several nodes as one child, such as a heading's words. */
  fragment(...children) {
    const fragment = this.document.createDocumentFragment();
    fragment.append(...this.nodes(children));
    return fragment;
  }
};

// src/adapters/fetched_store/store_format.ts
var STORE_FORMAT = 1;
function mismatch(what) {
  throw new TypeError(`the site's data holds no ${what} where one is read`);
}
var text = (data) => typeof data === "string" ? data : mismatch("text");
var truth = (data) => typeof data === "boolean" ? data : mismatch("true or false");
var whole = (data) => typeof data === "number" && Number.isSafeInteger(data) && data >= 0 ? data : mismatch("whole number");
var format = (data) => whole(data) === STORE_FORMAT ? STORE_FORMAT : mismatch(`format ${STORE_FORMAT}`);
function list(item) {
  return (data) => Array.isArray(data) ? data.map(item) : mismatch("list");
}
function nullable(value) {
  return (data) => data === null ? null : value(data);
}
function oneOf(...values2) {
  return (data) => values2.find((value) => value === data) ?? mismatch(values2.join(" or "));
}
function object(members) {
  return (data) => {
    if (typeof data !== "object" || data === null || Array.isArray(data)) {
      return mismatch("object");
    }
    const found2 = new Map(Object.entries(data));
    const read = Object.entries(members).map(([key, member]) => [key, member(found2.get(key))]);
    return Object.fromEntries(read);
  };
}
function later(reader) {
  return (data) => reader()(data);
}
var table = nullable(list(list(text)));
var part = object({
  kind: oneOf("text", "parameter", "number", "placeholder"),
  text,
  parts: later(() => list(part))
});
var docString = object({ media_type: nullable(text), content: text });
var step = object({
  keyword: text,
  text,
  kind: oneOf("given", "when", "then"),
  sentence: text,
  slots: list(text),
  parts: list(part),
  table,
  doc_string: nullable(docString)
});
var run = object({ anchor: text, steps: list(step) });
var block = object({
  kind: oneOf("feature", "background", "rule", "scenario", "scenario-outline", "examples"),
  keyword: text,
  note: truth,
  name: text,
  description: text,
  tags: list(text),
  anchor: text,
  hash: text,
  blocks: later(() => list(block)),
  steps: list(step),
  examples: later(() => list(block)),
  place: nullable(whole),
  table,
  runs: list(run)
});
var readObject = object({ format, language: text, lang: text, feature: block });
var featureEntry2 = object({ path: text, page: text, object: text, lines: list(whole), source: nullable(text) });
var tagLink = object({ address: text, text });
var tagEntry = object({ tag: text, page: text, link: nullable(tagLink) });
var startStates = object({ collapse_rules: truth, collapse_scenarios: truth, show_tree: truth, inline_backgrounds: truth });
var sentences = object({ given: list(text), when: list(text), then: list(text) });
var readDescriptor = object({
  format,
  ghg: text,
  version: nullable(text),
  preview: nullable(text),
  title: text,
  only: truth,
  start: startStates,
  features: list(featureEntry2),
  tags: list(tagEntry),
  sentences
});
var readVersions = object({
  format,
  development: list(text),
  releases: list(text),
  pre_releases: list(text),
  precedence: list(list(text))
});
var pageObject = object({ object: text, versions: list(text) });
var pageEntry = object({ page: text, objects: list(pageObject) });
var readPages = object({ format, pages: list(pageEntry) });

// src/adapters/fetched_store.ts
var FetchedStore = class {
  root;
  descriptorPath;
  fetcher;
  kept = /* @__PURE__ */ new Map();
  /** `root` is the address of `data/`, and `descriptorPath` the descriptor's below it. Chromium, through Playwright 1.63.0, refuses `fetch` called as a method of another object, so it is called on its own. */
  constructor(root, descriptorPath2, fetcher = (input, init) => fetch(input, init)) {
    this.root = root;
    this.descriptorPath = descriptorPath2;
    this.fetcher = fetcher;
  }
  descriptor() {
    return this.file(this.descriptorPath, readDescriptor);
  }
  object(name2) {
    return this.file(`objects/${encodeURIComponent(name2)}.json`, readObject);
  }
  versions() {
    return this.file("versions.json", readVersions);
  }
  pages() {
    return this.file("pages.json", readPages);
  }
  /** A file that cannot be fetched or read is forgotten, so that it is fetched again when next asked for. */
  file(path, read) {
    const known = this.kept.get(path);
    if (known !== void 0) {
      return known;
    }
    const fetched = this.fetched(path, read);
    this.kept.set(path, fetched);
    fetched.catch(() => this.kept.delete(path));
    return fetched;
  }
  async fetched(path, read) {
    const address = new URL(path, this.root);
    const response = await this.fetcher(address);
    if (!response.ok) {
      throw new Error(`${address.href}: ${response.status}`);
    }
    return read(await response.json());
  }
};

// node_modules/@fluent/bundle/esm/types.js
var FluentType = class {
  /**
   * Create a `FluentType` instance.
   *
   * @param value The JavaScript value to wrap.
   */
  constructor(value) {
    this.value = value;
  }
  /**
   * Unwrap the raw value stored by this `FluentType`.
   */
  valueOf() {
    return this.value;
  }
};
var FluentNone = class extends FluentType {
  /**
   * Create an instance of `FluentNone` with an optional fallback value.
   * @param value The fallback value of this `FluentNone`.
   */
  constructor(value = "???") {
    super(value);
  }
  /**
   * Format this `FluentNone` to the fallback string.
   */
  toString(scope) {
    return `{${this.value}}`;
  }
};
var FluentNumber = class extends FluentType {
  /**
   * Create an instance of `FluentNumber` with options to the
   * `Intl.NumberFormat` constructor.
   *
   * @param value The number value of this `FluentNumber`.
   * @param opts Options which will be passed to `Intl.NumberFormat`.
   */
  constructor(value, opts = {}) {
    super(value);
    this.opts = opts;
  }
  /**
   * Format this `FluentNumber` to a string.
   */
  toString(scope) {
    if (scope) {
      try {
        const nf = scope.memoizeIntlObject(Intl.NumberFormat, this.opts);
        return nf.format(this.value);
      } catch (err) {
        scope.reportError(err);
      }
    }
    return this.value.toString(10);
  }
};
var FluentDateTime = class _FluentDateTime extends FluentType {
  static supportsValue(value) {
    if (typeof value === "number")
      return true;
    if (value instanceof Date)
      return true;
    if (value instanceof FluentType)
      return _FluentDateTime.supportsValue(value.valueOf());
    if ("Temporal" in globalThis) {
      const _Temporal = globalThis.Temporal;
      if (value instanceof _Temporal.Instant || value instanceof _Temporal.PlainDateTime || value instanceof _Temporal.PlainDate || value instanceof _Temporal.PlainMonthDay || value instanceof _Temporal.PlainTime || value instanceof _Temporal.PlainYearMonth) {
        return true;
      }
    }
    return false;
  }
  /**
   * Create an instance of `FluentDateTime` with options to the
   * `Intl.DateTimeFormat` constructor.
   *
   * @param value The number value of this `FluentDateTime`, in milliseconds.
   * @param opts Options which will be passed to `Intl.DateTimeFormat`.
   */
  constructor(value, opts = {}) {
    if (value instanceof _FluentDateTime) {
      opts = { ...value.opts, ...opts };
      value = value.value;
    } else if (value instanceof FluentType) {
      value = value.valueOf();
    }
    if (typeof value === "object" && "calendarId" in value && opts.calendar === void 0) {
      opts = { ...opts, calendar: value.calendarId };
    }
    super(value);
    this.opts = opts;
  }
  [Symbol.toPrimitive](hint) {
    return hint === "string" ? this.toString() : this.toNumber();
  }
  /**
   * Convert this `FluentDateTime` to a number.
   * Note that this isn't always possible due to the nature of Temporal objects.
   * In such cases, a TypeError will be thrown.
   */
  toNumber() {
    const value = this.value;
    if (typeof value === "number")
      return value;
    if (value instanceof Date)
      return value.getTime();
    if ("epochMilliseconds" in value) {
      return value.epochMilliseconds;
    }
    if ("toZonedDateTime" in value) {
      return value.toZonedDateTime("UTC").epochMilliseconds;
    }
    throw new TypeError("Unwrapping a non-number value as a number");
  }
  /**
   * Format this `FluentDateTime` to a string.
   */
  toString(scope) {
    if (scope) {
      try {
        const dtf = scope.memoizeIntlObject(Intl.DateTimeFormat, this.opts);
        return dtf.format(this.value);
      } catch (err) {
        scope.reportError(err);
      }
    }
    if (typeof this.value === "number" || this.value instanceof Date) {
      return new Date(this.value).toISOString();
    }
    return this.value.toString();
  }
};

// node_modules/@fluent/bundle/esm/resolver.js
var MAX_PLACEABLES = 100;
var FSI = "⁨";
var PDI = "⁩";
function match(scope, selector, key) {
  if (key === selector) {
    return true;
  }
  if (key instanceof FluentNumber && selector instanceof FluentNumber && key.value === selector.value) {
    return true;
  }
  if (selector instanceof FluentNumber && typeof key === "string") {
    let category = scope.memoizeIntlObject(Intl.PluralRules, selector.opts).select(selector.value);
    if (key === category) {
      return true;
    }
  }
  return false;
}
function getDefault(scope, variants, star) {
  if (variants[star]) {
    return resolvePattern(scope, variants[star].value);
  }
  scope.reportError(new RangeError("No default"));
  return new FluentNone();
}
function getArguments(scope, args) {
  const positional = [];
  const named3 = /* @__PURE__ */ Object.create(null);
  for (const arg of args) {
    if (arg.type === "narg") {
      named3[arg.name] = resolveExpression(scope, arg.value);
    } else {
      positional.push(resolveExpression(scope, arg));
    }
  }
  return { positional, named: named3 };
}
function resolveExpression(scope, expr) {
  switch (expr.type) {
    case "str":
      return expr.value;
    case "num":
      return new FluentNumber(expr.value, {
        minimumFractionDigits: expr.precision
      });
    case "var":
      return resolveVariableReference(scope, expr);
    case "mesg":
      return resolveMessageReference(scope, expr);
    case "term":
      return resolveTermReference(scope, expr);
    case "func":
      return resolveFunctionReference(scope, expr);
    case "select":
      return resolveSelectExpression(scope, expr);
    default:
      return new FluentNone();
  }
}
function resolveVariableReference(scope, { name: name2 }) {
  let arg;
  if (scope.params) {
    if (Object.prototype.hasOwnProperty.call(scope.params, name2)) {
      arg = scope.params[name2];
    } else {
      return new FluentNone(`$${name2}`);
    }
  } else if (scope.args && Object.prototype.hasOwnProperty.call(scope.args, name2)) {
    arg = scope.args[name2];
  } else {
    scope.reportError(new ReferenceError(`Unknown variable: $${name2}`));
    return new FluentNone(`$${name2}`);
  }
  if (arg instanceof FluentType) {
    return arg;
  }
  switch (typeof arg) {
    case "string":
      return arg;
    case "number":
      return new FluentNumber(arg);
    case "object":
      if (FluentDateTime.supportsValue(arg)) {
        return new FluentDateTime(arg);
      }
    // eslint-disable-next-line no-fallthrough
    default:
      scope.reportError(new TypeError(`Variable type not supported: $${name2}, ${typeof arg}`));
      return new FluentNone(`$${name2}`);
  }
}
function resolveMessageReference(scope, { name: name2, attr }) {
  const message = scope.bundle._messages.get(name2);
  if (!message) {
    scope.reportError(new ReferenceError(`Unknown message: ${name2}`));
    return new FluentNone(name2);
  }
  if (attr) {
    const attribute = message.attributes[attr];
    if (attribute) {
      return resolvePattern(scope, attribute);
    }
    scope.reportError(new ReferenceError(`Unknown attribute: ${attr}`));
    return new FluentNone(`${name2}.${attr}`);
  }
  if (message.value) {
    return resolvePattern(scope, message.value);
  }
  scope.reportError(new ReferenceError(`No value: ${name2}`));
  return new FluentNone(name2);
}
function resolveTermReference(scope, { name: name2, attr, args }) {
  const id = `-${name2}`;
  const term = scope.bundle._terms.get(id);
  if (!term) {
    scope.reportError(new ReferenceError(`Unknown term: ${id}`));
    return new FluentNone(id);
  }
  if (attr) {
    const attribute = term.attributes[attr];
    if (attribute) {
      scope.params = getArguments(scope, args).named;
      const resolved2 = resolvePattern(scope, attribute);
      scope.params = null;
      return resolved2;
    }
    scope.reportError(new ReferenceError(`Unknown attribute: ${attr}`));
    return new FluentNone(`${id}.${attr}`);
  }
  scope.params = getArguments(scope, args).named;
  const resolved = resolvePattern(scope, term.value);
  scope.params = null;
  return resolved;
}
function resolveFunctionReference(scope, { name: name2, args }) {
  let func = scope.bundle._functions[name2];
  if (!func) {
    scope.reportError(new ReferenceError(`Unknown function: ${name2}()`));
    return new FluentNone(`${name2}()`);
  }
  if (typeof func !== "function") {
    scope.reportError(new TypeError(`Function ${name2}() is not callable`));
    return new FluentNone(`${name2}()`);
  }
  try {
    let resolved = getArguments(scope, args);
    return func(resolved.positional, resolved.named);
  } catch (err) {
    scope.reportError(err);
    return new FluentNone(`${name2}()`);
  }
}
function resolveSelectExpression(scope, { selector, variants, star }) {
  let sel = resolveExpression(scope, selector);
  if (sel instanceof FluentNone) {
    return getDefault(scope, variants, star);
  }
  for (const variant of variants) {
    const key = resolveExpression(scope, variant.key);
    if (match(scope, sel, key)) {
      return resolvePattern(scope, variant.value);
    }
  }
  return getDefault(scope, variants, star);
}
function resolveComplexPattern(scope, ptn) {
  if (scope.dirty.has(ptn)) {
    scope.reportError(new RangeError("Cyclic reference"));
    return new FluentNone();
  }
  scope.dirty.add(ptn);
  const result2 = [];
  const useIsolating = scope.bundle._useIsolating && ptn.length > 1;
  for (const elem of ptn) {
    if (typeof elem === "string") {
      result2.push(scope.bundle._transform(elem));
      continue;
    }
    scope.placeables++;
    if (scope.placeables > MAX_PLACEABLES) {
      scope.dirty.delete(ptn);
      throw new RangeError(`Too many placeables expanded: ${scope.placeables}, max allowed is ${MAX_PLACEABLES}`);
    }
    if (useIsolating) {
      result2.push(FSI);
    }
    result2.push(resolveExpression(scope, elem).toString(scope));
    if (useIsolating) {
      result2.push(PDI);
    }
  }
  scope.dirty.delete(ptn);
  return result2.join("");
}
function resolvePattern(scope, value) {
  if (typeof value === "string") {
    return scope.bundle._transform(value);
  }
  return resolveComplexPattern(scope, value);
}

// node_modules/@fluent/bundle/esm/scope.js
var Scope = class {
  constructor(bundle, errors, args) {
    this.dirty = /* @__PURE__ */ new WeakSet();
    this.params = null;
    this.placeables = 0;
    this.bundle = bundle;
    this.errors = errors;
    this.args = args;
  }
  reportError(error) {
    if (!this.errors || !(error instanceof Error)) {
      throw error;
    }
    this.errors.push(error);
  }
  memoizeIntlObject(ctor, opts) {
    let cache2 = this.bundle._intls.get(ctor);
    if (!cache2) {
      cache2 = {};
      this.bundle._intls.set(ctor, cache2);
    }
    let id = JSON.stringify(opts);
    if (!cache2[id]) {
      cache2[id] = new ctor(this.bundle.locales, opts);
    }
    return cache2[id];
  }
};

// node_modules/@fluent/bundle/esm/builtins.js
function values(opts, allowed) {
  const unwrapped = /* @__PURE__ */ Object.create(null);
  for (const [name2, opt] of Object.entries(opts)) {
    if (allowed.includes(name2)) {
      unwrapped[name2] = opt.valueOf();
    }
  }
  return unwrapped;
}
var NUMBER_ALLOWED = [
  "unitDisplay",
  "currencyDisplay",
  "useGrouping",
  "minimumIntegerDigits",
  "minimumFractionDigits",
  "maximumFractionDigits",
  "minimumSignificantDigits",
  "maximumSignificantDigits"
];
function NUMBER(args, opts) {
  let arg = args[0];
  if (arg instanceof FluentNone) {
    return new FluentNone(`NUMBER(${arg.valueOf()})`);
  }
  if (arg instanceof FluentNumber) {
    return new FluentNumber(arg.valueOf(), {
      ...arg.opts,
      ...values(opts, NUMBER_ALLOWED)
    });
  }
  if (arg instanceof FluentDateTime) {
    return new FluentNumber(arg.toNumber(), {
      ...values(opts, NUMBER_ALLOWED)
    });
  }
  throw new TypeError("Invalid argument to NUMBER");
}
var DATETIME_ALLOWED = [
  "dateStyle",
  "timeStyle",
  "fractionalSecondDigits",
  "dayPeriod",
  "hour12",
  "weekday",
  "era",
  "year",
  "month",
  "day",
  "hour",
  "minute",
  "second",
  "timeZoneName"
];
function DATETIME(args, opts) {
  let arg = args[0];
  if (arg instanceof FluentNone) {
    return new FluentNone(`DATETIME(${arg.valueOf()})`);
  }
  if (arg instanceof FluentDateTime || arg instanceof FluentNumber) {
    return new FluentDateTime(arg, values(opts, DATETIME_ALLOWED));
  }
  throw new TypeError("Invalid argument to DATETIME");
}

// node_modules/@fluent/bundle/esm/memoizer.js
var cache = /* @__PURE__ */ new Map();
function getMemoizerForLocale(locales) {
  const stringLocale = Array.isArray(locales) ? locales.join(" ") : locales;
  let memoizer = cache.get(stringLocale);
  if (memoizer === void 0) {
    memoizer = /* @__PURE__ */ new Map();
    cache.set(stringLocale, memoizer);
  }
  return memoizer;
}

// node_modules/@fluent/bundle/esm/bundle.js
var FluentBundle = class {
  /**
   * Create an instance of `FluentBundle`.
   *
   * @example
   * ```js
   * let bundle = new FluentBundle(["en-US", "en"]);
   *
   * let bundle = new FluentBundle(locales, {useIsolating: false});
   *
   * let bundle = new FluentBundle(locales, {
   *   useIsolating: true,
   *   functions: {
   *     NODE_ENV: () => process.env.NODE_ENV
   *   }
   * });
   * ```
   *
   * @param locales - Used to instantiate `Intl` formatters used by translations.
   * @param options - Optional configuration for the bundle.
   */
  constructor(locales, { functions, useIsolating = true, transform = (v) => v } = {}) {
    this._terms = /* @__PURE__ */ new Map();
    this._messages = /* @__PURE__ */ new Map();
    this.locales = Array.isArray(locales) ? locales : [locales];
    this._functions = {
      NUMBER,
      DATETIME,
      ...functions
    };
    this._useIsolating = useIsolating;
    this._transform = transform;
    this._intls = getMemoizerForLocale(locales);
  }
  /**
   * Check if a message is present in the bundle.
   *
   * @param id - The identifier of the message to check.
   */
  hasMessage(id) {
    return this._messages.has(id);
  }
  /**
   * Return a raw unformatted message object from the bundle.
   *
   * Raw messages are `{value, attributes}` shapes containing translation units
   * called `Patterns`. `Patterns` are implementation-specific; they should be
   * treated as black boxes and formatted with `FluentBundle.formatPattern`.
   *
   * @param id - The identifier of the message to check.
   */
  getMessage(id) {
    return this._messages.get(id);
  }
  /**
   * Add a translation resource to the bundle.
   *
   * @example
   * ```js
   * let res = new FluentResource("foo = Foo");
   * bundle.addResource(res);
   * bundle.getMessage("foo");
   * // → {value: .., attributes: {..}}
   * ```
   *
   * @param res
   * @param options
   */
  addResource(res, { allowOverrides = false } = {}) {
    const errors = [];
    for (let i = 0; i < res.body.length; i++) {
      let entry = res.body[i];
      if (entry.id.startsWith("-")) {
        if (allowOverrides === false && this._terms.has(entry.id)) {
          errors.push(new Error(`Attempt to override an existing term: "${entry.id}"`));
          continue;
        }
        this._terms.set(entry.id, entry);
      } else {
        if (allowOverrides === false && this._messages.has(entry.id)) {
          errors.push(new Error(`Attempt to override an existing message: "${entry.id}"`));
          continue;
        }
        this._messages.set(entry.id, entry);
      }
    }
    return errors;
  }
  /**
   * Format a `Pattern` to a string.
   *
   * Format a raw `Pattern` into a string. `args` will be used to resolve
   * references to variables passed as arguments to the translation.
   *
   * In case of errors `formatPattern` will try to salvage as much of the
   * translation as possible and will still return a string. For performance
   * reasons, the encountered errors are not returned but instead are appended
   * to the `errors` array passed as the third argument.
   *
   * If `errors` is omitted, the first encountered error will be thrown.
   *
   * @example
   * ```js
   * let errors = [];
   * bundle.addResource(
   *     new FluentResource("hello = Hello, {$name}!"));
   *
   * let hello = bundle.getMessage("hello");
   * if (hello.value) {
   *     bundle.formatPattern(hello.value, {name: "Jane"}, errors);
   *     // Returns "Hello, Jane!" and `errors` is empty.
   *
   *     bundle.formatPattern(hello.value, undefined, errors);
   *     // Returns "Hello, {$name}!" and `errors` is now:
   *     // [<ReferenceError: Unknown variable: name>]
   * }
   * ```
   */
  formatPattern(pattern, args = null, errors = null) {
    if (typeof pattern === "string") {
      return this._transform(pattern);
    }
    let scope = new Scope(this, errors, args);
    try {
      let value = resolveComplexPattern(scope, pattern);
      return value.toString(scope);
    } catch (err) {
      if (scope.errors && err instanceof Error) {
        scope.errors.push(err);
        return new FluentNone().toString(scope);
      }
      throw err;
    }
  }
};

// node_modules/@fluent/bundle/esm/resource.js
var RE_MESSAGE_START = /^(-?[a-zA-Z][\w-]*) *= */gm;
var RE_ATTRIBUTE_START = /\.([a-zA-Z][\w-]*) *= */y;
var RE_VARIANT_START = /\*?\[/y;
var RE_NUMBER_LITERAL = /(-?[0-9]+(?:\.([0-9]+))?)/y;
var RE_IDENTIFIER = /([a-zA-Z][\w-]*)/y;
var RE_REFERENCE = /([$-])?([a-zA-Z][\w-]*)(?:\.([a-zA-Z][\w-]*))?/y;
var RE_FUNCTION_NAME = /^[A-Z][A-Z0-9_-]*$/;
var RE_TEXT_RUN = /([^{}\n\r]+)/y;
var RE_STRING_RUN = /([^\\"\n\r]*)/y;
var RE_STRING_ESCAPE = /\\([\\"])/y;
var RE_UNICODE_ESCAPE = /\\u([a-fA-F0-9]{4})|\\U([a-fA-F0-9]{6})/y;
var RE_LEADING_NEWLINES = /^\n+/;
var RE_TRAILING_SPACES = / +$/;
var RE_BLANK_LINES = / *\r?\n/g;
var RE_INDENT = /( *)$/;
var TOKEN_BRACE_OPEN = /{\s*/y;
var TOKEN_BRACE_CLOSE = /\s*}/y;
var TOKEN_BRACKET_OPEN = /\[\s*/y;
var TOKEN_BRACKET_CLOSE = /\s*] */y;
var TOKEN_PAREN_OPEN = /\s*\(\s*/y;
var TOKEN_ARROW = /\s*->\s*/y;
var TOKEN_COLON = /\s*:\s*/y;
var TOKEN_COMMA = /\s*,?\s*/y;
var TOKEN_BLANK = /\s+/y;
var FluentResource = class {
  constructor(source) {
    this.body = [];
    RE_MESSAGE_START.lastIndex = 0;
    let cursor = 0;
    while (true) {
      let next = RE_MESSAGE_START.exec(source);
      if (next === null) {
        break;
      }
      cursor = RE_MESSAGE_START.lastIndex;
      try {
        this.body.push(parseMessage(next[1]));
      } catch (err) {
        if (err instanceof SyntaxError) {
          continue;
        }
        throw err;
      }
    }
    function test(re) {
      re.lastIndex = cursor;
      return re.test(source);
    }
    function consumeChar(char, errorClass) {
      if (source[cursor] === char) {
        cursor++;
        return true;
      }
      if (errorClass) {
        throw new errorClass(`Expected ${char}`);
      }
      return false;
    }
    function consumeToken(re, errorClass) {
      if (test(re)) {
        cursor = re.lastIndex;
        return true;
      }
      if (errorClass) {
        throw new errorClass(`Expected ${re.toString()}`);
      }
      return false;
    }
    function match2(re) {
      re.lastIndex = cursor;
      let result2 = re.exec(source);
      if (result2 === null) {
        throw new SyntaxError(`Expected ${re.toString()}`);
      }
      cursor = re.lastIndex;
      return result2;
    }
    function match1(re) {
      return match2(re)[1];
    }
    function parseMessage(id) {
      let value = parsePattern();
      let attributes = parseAttributes();
      if (value === null && Object.keys(attributes).length === 0) {
        throw new SyntaxError("Expected message value or attributes");
      }
      return { id, value, attributes };
    }
    function parseAttributes() {
      let attrs = /* @__PURE__ */ Object.create(null);
      while (test(RE_ATTRIBUTE_START)) {
        let name2 = match1(RE_ATTRIBUTE_START);
        let value = parsePattern();
        if (value === null) {
          throw new SyntaxError("Expected attribute value");
        }
        attrs[name2] = value;
      }
      return attrs;
    }
    function parsePattern() {
      let first;
      if (test(RE_TEXT_RUN)) {
        first = match1(RE_TEXT_RUN);
      }
      if (source[cursor] === "{" || source[cursor] === "}") {
        return parsePatternElements(first ? [first] : [], Infinity);
      }
      let indent = parseIndent();
      if (indent) {
        if (first) {
          return parsePatternElements([first, indent], indent.length);
        }
        indent.value = trim(indent.value, RE_LEADING_NEWLINES);
        return parsePatternElements([indent], indent.length);
      }
      if (first) {
        return trim(first, RE_TRAILING_SPACES);
      }
      return null;
    }
    function parsePatternElements(elements = [], commonIndent) {
      while (true) {
        if (test(RE_TEXT_RUN)) {
          elements.push(match1(RE_TEXT_RUN));
          continue;
        }
        if (source[cursor] === "{") {
          elements.push(parsePlaceable());
          continue;
        }
        if (source[cursor] === "}") {
          throw new SyntaxError("Unbalanced closing brace");
        }
        let indent = parseIndent();
        if (indent) {
          elements.push(indent);
          commonIndent = Math.min(commonIndent, indent.length);
          continue;
        }
        break;
      }
      let lastIndex = elements.length - 1;
      let lastElement = elements[lastIndex];
      if (typeof lastElement === "string") {
        elements[lastIndex] = trim(lastElement, RE_TRAILING_SPACES);
      }
      let baked = [];
      for (let element of elements) {
        if (element instanceof Indent) {
          element = element.value.slice(0, element.value.length - commonIndent);
        }
        if (element) {
          baked.push(element);
        }
      }
      return baked;
    }
    function parsePlaceable() {
      consumeToken(TOKEN_BRACE_OPEN, SyntaxError);
      let selector = parseInlineExpression();
      if (consumeToken(TOKEN_BRACE_CLOSE)) {
        return selector;
      }
      if (consumeToken(TOKEN_ARROW)) {
        let variants = parseVariants();
        consumeToken(TOKEN_BRACE_CLOSE, SyntaxError);
        return {
          type: "select",
          selector,
          ...variants
        };
      }
      throw new SyntaxError("Unclosed placeable");
    }
    function parseInlineExpression() {
      if (source[cursor] === "{") {
        return parsePlaceable();
      }
      if (test(RE_REFERENCE)) {
        let [, sigil, name2, attr = null] = match2(RE_REFERENCE);
        if (sigil === "$") {
          return { type: "var", name: name2 };
        }
        if (consumeToken(TOKEN_PAREN_OPEN)) {
          let args = parseArguments();
          if (sigil === "-") {
            return { type: "term", name: name2, attr, args };
          }
          if (RE_FUNCTION_NAME.test(name2)) {
            return { type: "func", name: name2, args };
          }
          throw new SyntaxError("Function names must be all upper-case");
        }
        if (sigil === "-") {
          return {
            type: "term",
            name: name2,
            attr,
            args: []
          };
        }
        return { type: "mesg", name: name2, attr };
      }
      return parseLiteral();
    }
    function parseArguments() {
      let args = [];
      while (true) {
        switch (source[cursor]) {
          case ")":
            cursor++;
            return args;
          case void 0:
            throw new SyntaxError("Unclosed argument list");
        }
        args.push(parseArgument());
        consumeToken(TOKEN_COMMA);
      }
    }
    function parseArgument() {
      let expr = parseInlineExpression();
      if (expr.type !== "mesg") {
        return expr;
      }
      if (consumeToken(TOKEN_COLON)) {
        return {
          type: "narg",
          name: expr.name,
          value: parseLiteral()
        };
      }
      return expr;
    }
    function parseVariants() {
      let variants = [];
      let count = 0;
      let star;
      while (test(RE_VARIANT_START)) {
        if (consumeChar("*")) {
          star = count;
        }
        let key = parseVariantKey();
        let value = parsePattern();
        if (value === null) {
          throw new SyntaxError("Expected variant value");
        }
        variants[count++] = { key, value };
      }
      if (count === 0) {
        return null;
      }
      if (star === void 0) {
        throw new SyntaxError("Expected default variant");
      }
      return { variants, star };
    }
    function parseVariantKey() {
      consumeToken(TOKEN_BRACKET_OPEN, SyntaxError);
      let key;
      if (test(RE_NUMBER_LITERAL)) {
        key = parseNumberLiteral();
      } else {
        key = {
          type: "str",
          value: match1(RE_IDENTIFIER)
        };
      }
      consumeToken(TOKEN_BRACKET_CLOSE, SyntaxError);
      return key;
    }
    function parseLiteral() {
      if (test(RE_NUMBER_LITERAL)) {
        return parseNumberLiteral();
      }
      if (source[cursor] === '"') {
        return parseStringLiteral();
      }
      throw new SyntaxError("Invalid expression");
    }
    function parseNumberLiteral() {
      let [, value, fraction = ""] = match2(RE_NUMBER_LITERAL);
      let precision = fraction.length;
      return {
        type: "num",
        value: parseFloat(value),
        precision
      };
    }
    function parseStringLiteral() {
      consumeChar('"', SyntaxError);
      let value = "";
      while (true) {
        value += match1(RE_STRING_RUN);
        if (source[cursor] === "\\") {
          value += parseEscapeSequence();
          continue;
        }
        if (consumeChar('"')) {
          return { type: "str", value };
        }
        throw new SyntaxError("Unclosed string literal");
      }
    }
    function parseEscapeSequence() {
      if (test(RE_STRING_ESCAPE)) {
        return match1(RE_STRING_ESCAPE);
      }
      if (test(RE_UNICODE_ESCAPE)) {
        let [, codepoint4, codepoint6] = match2(RE_UNICODE_ESCAPE);
        let codepoint = parseInt(codepoint4 || codepoint6, 16);
        return codepoint <= 55295 || 57344 <= codepoint ? (
          // It's a Unicode scalar value.
          String.fromCodePoint(codepoint)
        ) : (
          // Lonely surrogates can cause trouble when the parsing result is
          // saved using UTF-8. Use U+FFFD REPLACEMENT CHARACTER instead.
          "�"
        );
      }
      throw new SyntaxError("Unknown escape sequence");
    }
    function parseIndent() {
      let start = cursor;
      consumeToken(TOKEN_BLANK);
      switch (source[cursor]) {
        case ".":
        case "[":
        case "*":
        case "}":
        case void 0:
          return false;
        case "{":
          return makeIndent(source.slice(start, cursor));
      }
      if (source[cursor - 1] === " ") {
        return makeIndent(source.slice(start, cursor));
      }
      return false;
    }
    function trim(text2, re) {
      return text2.replace(re, "");
    }
    function makeIndent(blank) {
      let value = blank.replace(RE_BLANK_LINES, "\n");
      let length = RE_INDENT.exec(blank)[1].length;
      return new Indent(value, length);
    }
  }
};
var Indent = class {
  constructor(value, length) {
    this.value = value;
    this.length = length;
  }
};

// src/adapters/fluent_words.ts
var MARK_BASE = 57344;
var FluentWords = class {
  bundle;
  /** `source` is the catalogue's messages, as a `.ftl` file holds them, in English, the only language for now. */
  constructor(source) {
    this.bundle = new FluentBundle("en", { useIsolating: false });
    this.bundle.addResource(new FluentResource(source));
  }
  text(id, args = {}) {
    const message = this.bundle.getMessage(id);
    if (message?.value == null) {
      return id;
    }
    const values2 = Object.fromEntries(Object.entries(args).map(([key, value]) => [key, typeof value === "number" ? new FluentNumber(value, { useGrouping: false }) : value]));
    return this.bundle.formatPattern(message.value, values2, []);
  }
  parts(id, args, texts) {
    const named3 = Object.entries(texts);
    const marks = named3.map((_, place) => String.fromCodePoint(MARK_BASE + place));
    const marked2 = Object.fromEntries(named3.map(([key], place) => [key, marks[place] ?? ""]));
    const words = this.text(id, { ...args, ...marked2 });
    const parts = [];
    let rest = words;
    for (; ; ) {
      const next = firstMark(rest, marks);
      if (next === null) {
        break;
      }
      parts.push({ text: rest.slice(0, next.at), isolated: false }, ...named3[next.place]?.[1] ?? []);
      rest = rest.slice(next.at + 1);
    }
    parts.push({ text: rest, isolated: false });
    return parts.filter((part2) => part2.text !== "");
  }
};
function firstMark(words, marks) {
  const found2 = marks.map((mark, place) => ({ at: words.indexOf(mark), place })).filter((mark) => mark.at >= 0);
  return found2.reduce((first, mark) => first === null || mark.at < first.at ? mark : first, null);
}

// src/adapters/dom_backgrounds.ts
var DomBackgrounds = class _DomBackgrounds {
  checkbox;
  document;
  constructor(document2, checkbox) {
    this.document = document2;
    this.checkbox = checkbox;
  }
  /** Null on a page without the checkbox, where no scenario starts with a background's steps. */
  static of(document2) {
    const checkbox = document2.querySelector("input.ghg-inline-backgrounds");
    return checkbox ? new _DomBackgrounds(document2, checkbox) : null;
  }
  writtenInline() {
    return this.checkbox.defaultChecked;
  }
  setInline(inline) {
    this.checkbox.checked = inline;
  }
  onSwitch(handler) {
    this.checkbox.addEventListener("change", () => handler(this.checkbox.checked));
  }
  onReturn(handler) {
    const listening = new AbortController();
    this.document.defaultView?.addEventListener(
      "pageshow",
      (event) => {
        if (!this.checkbox.isConnected) {
          listening.abort();
        } else if (event.persisted) {
          handler();
        }
      },
      { signal: listening.signal }
    );
  }
};

// src/adapters/dom_note_dialog.ts
var DomNoteDialog = class _DomNoteDialog {
  dialog;
  openers;
  constructor(dialog, openers) {
    this.dialog = dialog;
    this.openers = openers;
  }
  /** Null on a page without the dialog, where no block's file writes another keyword. */
  static of(document2) {
    const dialog = document2.querySelector("dialog#ghg_note_dialog");
    return dialog ? new _DomNoteDialog(dialog, [...document2.querySelectorAll("button.ghg-note-opener")]) : null;
  }
  onOpen(handler) {
    this.openers.forEach((button) => button.addEventListener("click", () => handler(button)));
  }
  open() {
    this.dialog.showModal();
    this.dialog.focus();
  }
  onClose(handler) {
    this.dialog.addEventListener("close", handler);
  }
};

// src/adapters/dom_page.ts
var DomBlock = class {
  anchor;
  parent;
  expandedWhenUnchecked;
  inScenario;
  button;
  content;
  constructor(anchor, button, content, parent, element) {
    this.anchor = anchor;
    this.button = button;
    this.content = content;
    this.parent = parent;
    this.expandedWhenUnchecked = element === null ? null : expandedWhenUnchecked(element);
    this.inScenario = element?.classList.contains("ghg-background-in-scenario") ?? false;
  }
  isExpanded() {
    return this.button.getAttribute("aria-expanded") === "true";
  }
  setExpanded(expanded) {
    this.button.setAttribute("aria-expanded", String(expanded));
    this.content.hidden = !expanded;
  }
};
var DomPage = class {
  found = [];
  byElement = /* @__PURE__ */ new Map();
  document;
  /** Whether the page catches the links to its anchors itself, as a page written whole does; a drawn page leaves them to the navigation. */
  followsLinks;
  /** `root` holds the blocks, the document's or a drawn page's `main`. */
  constructor(document2, root = document2, followsLinks = true) {
    this.document = document2;
    this.followsLinks = followsLinks;
    for (const element of root.querySelectorAll(".ghg-block")) {
      const heading2 = element.querySelector(":scope > :is(h2, h3, h4, h5)") ?? element;
      const button = heading2.querySelector(":scope > button.ghg-toggle");
      const content = element.querySelector(":scope > .ghg-content");
      if (!button || !content) {
        continue;
      }
      const around = element.parentElement?.closest(".ghg-block");
      const parent = around ? this.byElement.get(around) ?? null : null;
      const block2 = new DomBlock(heading2.id, button, content, parent, element);
      this.byElement.set(element, block2);
      this.found.push(block2);
    }
    for (const element of root.querySelectorAll(".ghg-feature")) {
      const link2 = element.querySelector(":scope > a[href]");
      const button = element.querySelector(":scope > button.ghg-toggle");
      const content = element.querySelector(":scope > .ghg-content");
      if (!link2 || !button || !content) {
        continue;
      }
      this.found.push(new DomBlock(link2.getAttribute("href") ?? "", button, content, null, null));
    }
  }
  blocks() {
    return this.found;
  }
  target() {
    return this.followsLinks ? anchorOf(this.document.location.hash) : null;
  }
  holder(anchor) {
    const named3 = this.found.find((block2) => block2.anchor === anchor);
    if (named3) {
      return named3;
    }
    const around = this.document.getElementById(anchor)?.closest(".ghg-block");
    return around ? this.byElement.get(around) ?? null : null;
  }
  onToggle(handler) {
    for (const block2 of this.found) {
      block2.button.addEventListener("click", () => handler(block2));
    }
  }
  onCollapseAll(handler) {
    this.onClick(".ghg-collapse-all", handler);
  }
  onExpandAll(handler) {
    this.onClick(".ghg-expand-all", handler);
  }
  onFollow(handler) {
    if (!this.followsLinks) {
      return;
    }
    const view = this.document.defaultView;
    view?.addEventListener("hashchange", () => {
      const anchor = anchorOf(this.document.location.hash);
      if (anchor !== null) {
        handler(anchor);
      }
    });
    this.document.addEventListener("click", (event) => {
      const link2 = followedLink(event);
      const anchor = link2 ? this.anchorOnPage(link2) : null;
      if (anchor !== null) {
        handler(anchor);
      }
    });
  }
  /** The anchor a link leads to on this very page, such as the folder tree's, whatever its address is relative to. */
  anchorOnPage(link2) {
    let address;
    try {
      address = new URL(link2.getAttribute("href") ?? "", this.document.location.href);
    } catch {
      return null;
    }
    const here = this.document.location;
    return address.origin === here.origin && address.pathname === here.pathname ? anchorOf(address.hash) : null;
  }
  onClick(selector, handler) {
    this.document.querySelector(selector)?.addEventListener("click", handler);
  }
};
function expandedWhenUnchecked(element) {
  switch (element.getAttribute("data-expanded-when-unchecked")) {
    case "true":
      return true;
    case "false":
      return false;
    default:
      return null;
  }
}
function anchorOf(hash) {
  if (!hash.startsWith("#") || hash.length === 1) {
    return null;
  }
  try {
    return decodeURIComponent(hash.slice(1));
  } catch {
    return hash.slice(1);
  }
}

// src/adapters/dom_search_page.ts
var DomSearchPage = class _DomSearchPage {
  document;
  heading;
  field;
  status;
  groups;
  searchedWords;
  /** Null when the page is not the page of search results, or not the whole of one; `searched`, when given, is the words of a shell's address. */
  static find(document2, searched = null) {
    const heading2 = document2.querySelector("h1.ghg-search-heading");
    const field = document2.querySelector("form.ghg-search input.ghg-search-field");
    const status2 = document2.querySelector(".ghg-results-status");
    const groups = document2.querySelector(".ghg-search-groups");
    if (heading2 && field && status2 && groups) {
      return new _DomSearchPage(document2, heading2, field, status2, groups, searched);
    }
    return null;
  }
  constructor(document2, heading2, field, status2, groups, searched) {
    this.searchedWords = searched;
    this.document = document2;
    this.heading = heading2;
    this.field = field;
    this.status = status2;
    this.groups = groups;
  }
  words() {
    const data = this.groups.dataset;
    const text2 = (key) => data[key] ?? "";
    return {
      heading: text2("heading"),
      title: text2("title"),
      loading: text2("loading"),
      failed: text2("failed"),
      none: text2("none"),
      count: text2("count"),
      within: text2("within"),
      separator: text2("separator"),
      matchingSteps: text2("matchingSteps"),
      matchingTags: text2("matchingTags"),
      matchingCells: text2("matchingCells"),
      headings: {
        feature: text2("feature"),
        unnamedFeature: text2("unnamedFeature"),
        rule: text2("rule"),
        unnamedRule: text2("unnamedRule"),
        background: text2("background"),
        unnamedBackground: text2("unnamedBackground"),
        scenario: text2("scenario"),
        unnamedScenario: text2("unnamedScenario"),
        scenarioOutline: text2("scenarioOutline"),
        unnamedScenarioOutline: text2("unnamedScenarioOutline"),
        examples: text2("examples"),
        numberedExamples: text2("numberedExamples")
      }
    };
  }
  searched() {
    if (this.searchedWords !== null) {
      return this.searchedWords;
    }
    const address = this.document.defaultView?.location.href ?? "";
    return new URL(address, "https://example.invalid/").searchParams.get("q") ?? "";
  }
  showSearched(words, heading2, title) {
    this.field.value = words;
    this.heading.replaceChildren(...this.parts(heading2));
    this.document.title = title;
  }
  setStatus(text2) {
    this.status.textContent = text2;
  }
  showGroups(groups) {
    const words = this.words();
    this.groups.replaceChildren(...groups.flatMap((group) => this.group(words, group)));
  }
  group(words, group) {
    const heading2 = this.element("h2", [this.link(group.address, group.heading)]);
    const list2 = this.element(
      "ul",
      group.results.map((result2) => this.element("li", this.result(words, result2)))
    );
    list2.className = "ghg-search-group";
    return [heading2, list2];
  }
  result(words, result2) {
    const parts = [this.link(result2.address, result2.heading)];
    if (result2.within !== null) {
      parts.push(this.element("p", this.parts(result2.within)));
    }
    if (result2.steps.length > 0) {
      parts.push(
        this.element("p", [words.matchingSteps]),
        this.element(
          "ul",
          result2.steps.map((step2) => this.element("li", [this.name(step2)]))
        )
      );
    }
    for (const [label2, texts] of [
      [words.matchingTags, result2.tags],
      [words.matchingCells, result2.cells]
    ]) {
      if (texts.length > 0) {
        const named3 = texts.flatMap((text2, at) => at === 0 ? [this.name(text2)] : [words.separator, this.name(text2)]);
        parts.push(this.element("p", [`${label2} `, ...named3]));
      }
    }
    return parts;
  }
  link(address, text2) {
    const link2 = this.element("a", this.parts(text2));
    link2.setAttribute("href", address);
    return link2;
  }
  /** Each part's runs, a name's isolated in a `bdi` of its own. */
  parts(parts) {
    return parts.flatMap((part2) => part2.isName ? [this.name(part2.runs)] : this.runs(part2.runs));
  }
  name(runs) {
    return this.element("bdi", this.runs(runs));
  }
  /** Each run as text, a marked one in a `mark` of its own. */
  runs(runs) {
    return runs.map((run2) => run2.marked ? this.element("mark", [run2.text]) : run2.text);
  }
  element(name2, children) {
    const element = this.document.createElement(name2);
    element.append(...children);
    return element;
  }
};

// src/adapters/dom_drawn_page.ts
var DomDrawnPage = class {
  document;
  canvas;
  constructor(document2, canvas) {
    this.document = document2;
    this.canvas = canvas;
  }
  blocks() {
    return new DomPage(this.document, this.canvas.main, false);
  }
  backgrounds() {
    return DomBackgrounds.of(this.document);
  }
  noteDialog() {
    return DomNoteDialog.of(this.document);
  }
  searchPage(searched) {
    return DomSearchPage.find(this.document, searched);
  }
};

// src/adapters/dom_search_box.ts
var REANNOUNCE_MILLISECONDS = 50;
var OPTION_ID = "ghg_search_result_";
var DomSearchBox = class _DomSearchBox {
  form;
  field;
  link;
  status;
  results;
  pendingStatus = null;
  /** Null when the page has no search form, or not the whole of one. */
  static find(document2) {
    const form = document2.querySelector("form.ghg-search");
    const field = form?.querySelector("input.ghg-search-field");
    const link2 = form?.querySelector("a.ghg-search-all");
    const status2 = form?.querySelector(".ghg-search-status");
    const results = form?.querySelector("ol.ghg-search-results");
    if (form && field && link2 && status2 && results && form.dataset["root"] !== void 0 && form.dataset["shown"] !== void 0) {
      return new _DomSearchBox(form, field, link2, status2, results);
    }
    return null;
  }
  constructor(form, field, link2, status2, results) {
    this.form = form;
    this.field = field;
    this.link = link2;
    this.status = status2;
    this.results = results;
    form.addEventListener("submit", (event) => event.preventDefault());
    results.addEventListener("mousedown", (event) => event.preventDefault());
  }
  words() {
    const data = this.form.dataset;
    const text2 = (key) => data[key] ?? "";
    return {
      shown: Number(data["shown"]),
      loading: text2("loading"),
      failed: text2("failed"),
      none: text2("none"),
      count: text2("count"),
      countLimited: text2("countLimited"),
      noName: text2("noName"),
      feature: text2("feature"),
      rule: text2("rule"),
      scenario: text2("scenario"),
      scenarioInRule: text2("scenarioInRule"),
      scenarioOutline: text2("scenarioOutline"),
      scenarioOutlineInRule: text2("scenarioOutlineInRule"),
      background: text2("background"),
      backgroundInRule: text2("backgroundInRule"),
      examples: text2("examples"),
      examplesInRule: text2("examplesInRule"),
      numberedExamples: text2("numberedExamples"),
      allResults: text2("allResults")
    };
  }
  onInput(handler) {
    this.field.addEventListener("input", () => handler(this.field.value));
  }
  onDown(handler) {
    this.onKey("ArrowDown", handler);
  }
  onUp(handler) {
    this.onKey("ArrowUp", handler);
  }
  onEnter(handler) {
    this.onKey("Enter", handler);
  }
  onEscape(handler) {
    this.onKey("Escape", handler);
  }
  onPick(handler) {
    this.results.addEventListener("click", (event) => {
      const option = event.target?.closest?.("[role=option]");
      const place = option ? [...this.results.children].indexOf(option) : -1;
      if (place >= 0) {
        handler(place);
      }
    });
  }
  setStatus(text2) {
    const view = this.form.ownerDocument.defaultView;
    if (this.pendingStatus !== null) {
      view?.clearTimeout(this.pendingStatus);
      this.pendingStatus = null;
    }
    if (text2 === "" || this.status.textContent !== text2 || view === null) {
      this.status.textContent = text2;
      return;
    }
    this.status.textContent = "";
    this.pendingStatus = view.setTimeout(() => {
      this.pendingStatus = null;
      this.status.textContent = text2;
    }, REANNOUNCE_MILLISECONDS);
  }
  showResults(results) {
    const document2 = this.form.ownerDocument;
    const options = results.map((result2, place) => {
      const option = document2.createElement("li");
      option.id = `${OPTION_ID}${place}`;
      option.setAttribute("role", "option");
      option.setAttribute("aria-selected", "false");
      option.append(...this.pieces(result2.label));
      return option;
    });
    this.results.replaceChildren(...options);
    this.choose(null);
  }
  setOpen(open) {
    this.results.hidden = !open;
    this.field.setAttribute("aria-expanded", String(open));
  }
  choose(place) {
    for (const [at, option2] of [...this.results.children].entries()) {
      option2.setAttribute("aria-selected", String(at === place));
    }
    const option = place === null ? null : this.results.children.item(place);
    if (option === null) {
      this.field.removeAttribute("aria-activedescendant");
      return;
    }
    this.field.setAttribute("aria-activedescendant", option.id);
    option.scrollIntoView?.({ block: "nearest" });
  }
  showAllResults(link2) {
    this.link.hidden = link2 === null;
    if (link2 === null) {
      this.link.replaceChildren();
      return;
    }
    this.link.replaceChildren(...this.pieces(link2.label));
    this.link.setAttribute("href", this.root() + link2.address);
  }
  fieldText() {
    return this.field.value;
  }
  fill(text2) {
    this.field.value = text2;
  }
  go(address) {
    this.form.ownerDocument.defaultView?.location.assign(this.root() + address);
  }
  root() {
    return this.form.dataset["root"] ?? "";
  }
  /** Each piece as text, a name isolated in a `bdi` of its own. */
  pieces(label2) {
    return label2.map((piece) => {
      if (!piece.isName) {
        return piece.text;
      }
      const name2 = this.form.ownerDocument.createElement("bdi");
      name2.textContent = piece.text;
      return name2;
    });
  }
  onKey(key, handler) {
    this.field.addEventListener("keydown", (event) => {
      if (event.key === key && !event.isComposing && event.keyCode !== 229 && !event.altKey && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();
        handler();
      }
    });
  }
};

// src/adapters/dom_theme.ts
var DomTheme = class {
  document;
  constructor(document2) {
    this.document = document2;
  }
  show(choice) {
    for (const radio of this.radios()) {
      radio.checked = radio.value === choice;
    }
  }
  apply(theme) {
    const root = this.document.documentElement;
    if (theme === null) {
      delete root.dataset.theme;
    } else {
      root.dataset.theme = theme;
    }
  }
  onChoose(handler) {
    for (const radio of this.radios()) {
      radio.addEventListener("change", () => {
        if (radio.checked && isChoice(radio.value)) {
          handler(radio.value);
        }
      });
    }
  }
  onRestore(handler) {
    this.document.defaultView?.addEventListener("pageshow", (event) => {
      if (event.persisted) {
        handler();
      }
    });
  }
  radios() {
    return [...this.document.querySelectorAll("input[name='ghg-theme']")].filter(
      (element) => element instanceof HTMLInputElement
    );
  }
};
function isChoice(value) {
  return value === "system" || value === "light" || value === "dark";
}

// src/policies/tree_state.ts
var NARROW = "not all and (min-width: 48em)";
var SHOWN = "shown";
var HIDDEN = "hidden";
var EXPANDED = "expanded";
var COLLAPSED = "collapsed";
function shownKey(sitePath) {
  return `gherkin-glass:${sitePath} tree`;
}
function itemKey(sitePath, address) {
  return `gherkin-glass:${sitePath} tree ${address}`;
}
function rememberedShown(shown) {
  return shown ? SHOWN : HIDDEN;
}
function rememberedExpanded(expanded) {
  return expanded ? EXPANDED : COLLAPSED;
}
function startChecked(configured, kept) {
  if (kept === SHOWN) {
    return true;
  }
  if (kept === HIDDEN) {
    return false;
  }
  return configured;
}
function treeShown(narrow, checked, open) {
  return narrow ? open : checked;
}
function focusHidden(place, narrow, shown) {
  switch (place) {
    case "tree":
      return !shown;
    case "checkbox":
      return narrow;
    case "button":
      return !narrow;
    case null:
      return false;
  }
}
function startExpanded(onPath, kept) {
  return onPath || kept === EXPANDED;
}
function pageAddress(sitePath, pagePath) {
  const site = decoded(sitePath);
  const page = decoded(pagePath);
  if (!page.startsWith(site)) {
    return null;
  }
  const address = page.slice(site.length);
  return address === "" || address.endsWith("/") ? `${address}index.html` : address;
}
function rootFrom(address) {
  return "../".repeat(address.split("/").length - 1);
}
function pathTo(items, address) {
  for (const item of items) {
    if (!item.address.includes("#") && decoded(item.address) === address) {
      return [item.address];
    }
    const inner = pathTo(item.items ?? [], address);
    if (inner.length > 0) {
      return [...inner, item.address];
    }
  }
  return [];
}
function readTree(data) {
  return isItems(data) ? data : null;
}
function isItems(data) {
  return Array.isArray(data) && data.every(isItem);
}
function isItem(data) {
  if (typeof data !== "object" || data === null) {
    return false;
  }
  const item = data;
  return (typeof item["text"] === "string" || Array.isArray(item["text"]) && item["text"].every((run2) => typeof run2 === "string")) && typeof item["address"] === "string" && (item["lang"] === void 0 || typeof item["lang"] === "string") && (item["items"] === void 0 || isItems(item["items"]));
}
function decoded(path) {
  try {
    return decodeURIComponent(path);
  } catch {
    return path;
  }
}

// src/adapters/dom_tree.ts
var DomTree = class _DomTree {
  document;
  landmark;
  /** What an item's address is relative to: the way back to the site's root, or the root itself. */
  root;
  nodes = [];
  byLink = /* @__PURE__ */ new Map();
  handler = () => {
  };
  /** `page` is the current page's address below the site's root, `site`, decoded, if it is below it. */
  constructor(document2, site, page) {
    this.document = document2;
    this.landmark = document2.querySelector("nav.ghg-tree");
    this.root = page === null ? site : rootFrom(page);
  }
  /** The tree of a shell, whose items' addresses are the shell's own, from its `#`. */
  static inShell(document2) {
    return new _DomTree(document2, new URL(document2.location.href), "");
  }
  build(items, current, expanded) {
    if (!this.landmark) {
      return;
    }
    this.nodes.length = 0;
    this.byLink.clear();
    const tree = this.document.createElement("ul");
    tree.setAttribute("role", "tree");
    tree.className = "ghg-treeview";
    tree.setAttribute("aria-label", this.landmark.getAttribute("aria-label") ?? "");
    this.add(tree, items, null, current, expanded);
    this.landmark.replaceChildren(tree);
    const stop = this.nodes.find((node) => node.address === current) ?? this.nodes[0];
    if (stop) {
      stop.link.tabIndex = 0;
    }
    tree.addEventListener("keydown", (event) => this.onKey(event));
  }
  onToggle(handler) {
    this.handler = handler;
  }
  /** `lang` is the feature's that holds `items`, if one does. */
  add(list2, items, parent, current, expanded, lang) {
    for (const item of items) {
      const itemLang = item.lang ?? lang;
      const entry = this.document.createElement("li");
      entry.setAttribute("role", "none");
      const link2 = this.document.createElement("a");
      link2.setAttribute("role", "treeitem");
      link2.href = typeof this.root === "string" ? this.root + item.address : new URL(item.address, this.root).href;
      link2.tabIndex = -1;
      if (item.address === current) {
        link2.setAttribute("aria-current", "page");
      }
      const inner = item.items ?? [];
      let group = null;
      if (inner.length > 0) {
        const marker = this.document.createElement("span");
        marker.className = "ghg-marker";
        marker.setAttribute("aria-hidden", "true");
        link2.append(marker);
        group = this.document.createElement("ul");
        group.setAttribute("role", "group");
        group.id = `ghg_tree_${this.nodes.length}`;
        group.setAttribute("aria-label", textOf(item));
        link2.setAttribute("aria-owns", group.id);
      }
      link2.append(...this.words(item, itemLang));
      entry.append(link2);
      const node = { link: link2, address: item.address, group, parent };
      this.nodes.push(node);
      this.byLink.set(link2, node);
      if (group) {
        entry.append(group);
        this.setExpanded(node, expanded(item.address));
        link2.querySelector(".ghg-marker")?.addEventListener("click", (event) => {
          event.preventDefault();
          event.stopPropagation();
          this.focus(node);
          this.toggle(node, !this.isExpanded(node));
        });
        this.add(group, inner, node, current, expanded, itemLang);
      }
      list2.append(entry);
    }
  }
  /**
   * The item's words, each run of feature text isolated in a `bdi` with its
   * feature's `lang`, so that the tree's own words are read in the page's.
   */
  words(item, lang) {
    if (typeof item.text === "string") {
      return [item.text];
    }
    return item.text.map((run2, place) => {
      if (place % 2 === 0) {
        return run2;
      }
      const isolated2 = this.document.createElement("bdi");
      isolated2.append(run2);
      if (lang !== void 0) {
        isolated2.lang = lang;
      }
      return isolated2;
    });
  }
  isExpanded(node) {
    return node.link.getAttribute("aria-expanded") === "true";
  }
  setExpanded(node, expanded) {
    if (node.group) {
      node.link.setAttribute("aria-expanded", String(expanded));
      node.group.hidden = !expanded;
    }
  }
  /** As the reader asked, and remembered. */
  toggle(node, expanded) {
    if (node.group && this.isExpanded(node) !== expanded) {
      this.setExpanded(node, expanded);
      this.handler(node.address, expanded);
    }
  }
  /** The items not inside a collapsed item, in the order of the tree. */
  visible() {
    return this.nodes.filter((node) => {
      for (let above = node.parent; above !== null; above = above.parent) {
        if (!this.isExpanded(above)) {
          return false;
        }
      }
      return true;
    });
  }
  focus(node) {
    for (const other of this.nodes) {
      other.link.tabIndex = other === node ? 0 : -1;
    }
    node.link.focus();
  }
  onKey(event) {
    if (event.altKey || event.ctrlKey || event.metaKey) {
      return;
    }
    const target = event.target;
    const node = target ? this.byLink.get(target) : void 0;
    if (!node) {
      return;
    }
    const visible = this.visible();
    const at = visible.indexOf(node);
    const move = (to) => {
      if (to) {
        this.focus(to);
      }
    };
    switch (event.key) {
      case "ArrowDown":
        move(visible[at + 1]);
        break;
      case "ArrowUp":
        move(visible[at - 1]);
        break;
      case "ArrowRight":
        if (node.group && !this.isExpanded(node)) {
          this.toggle(node, true);
        } else if (node.group) {
          move(visible[at + 1]);
        }
        break;
      case "ArrowLeft":
        if (node.group && this.isExpanded(node)) {
          this.toggle(node, false);
        } else if (node.parent) {
          move(node.parent);
        }
        break;
      case "Home":
        move(visible[0]);
        break;
      case "End":
        move(visible[visible.length - 1]);
        break;
      case " ":
        node.link.click();
        break;
      case "*":
        for (const sibling of this.nodes) {
          if (sibling.parent === node.parent) {
            this.toggle(sibling, true);
          }
        }
        break;
      default:
        if ([...event.key].length === 1 && /^[\p{L}\p{N}]$/u.test(event.key)) {
          const wanted = event.key.toLocaleLowerCase();
          const after = [...visible.slice(at + 1), ...visible.slice(0, at + 1)];
          move(after.find((other) => (other.link.textContent ?? "").trim().toLocaleLowerCase().startsWith(wanted)));
          break;
        }
        return;
    }
    event.preventDefault();
    event.stopPropagation();
  }
};
function textOf(item) {
  return typeof item.text === "string" ? item.text : item.text.join("");
}

// src/adapters/dom_tree_switch.ts
var DomTreeSwitch = class {
  checkbox;
  button;
  landmark;
  /**
   * The control the stylesheet hid with the focus on it, which the browser
   * then dropped to the body, often before the resize is reported.
   */
  dropped = null;
  constructor(document2) {
    this.checkbox = document2.querySelector("input.ghg-show-tree");
    this.button = document2.querySelector("button.ghg-tree-button");
    this.landmark = document2.querySelector("nav.ghg-tree");
    this.keepDropped(this.checkbox, "checkbox");
    this.keepDropped(this.button, "button");
    document2.addEventListener("focusin", () => {
      this.dropped = null;
    });
  }
  configured() {
    return this.checkbox?.checked ?? false;
  }
  setChecked(checked) {
    if (this.checkbox) {
      this.checkbox.checked = checked;
    }
  }
  setOpen(open) {
    this.button?.setAttribute("aria-expanded", String(open));
  }
  setShown(shown) {
    if (this.landmark) {
      this.landmark.hidden = !shown;
    }
  }
  disable() {
    if (this.checkbox) {
      this.checkbox.disabled = true;
    }
    if (this.button) {
      this.button.disabled = true;
    }
  }
  onChange(handler) {
    const checkbox = this.checkbox;
    checkbox?.addEventListener("change", () => handler(checkbox.checked));
  }
  onPress(handler) {
    this.button?.addEventListener("click", () => handler());
  }
  onEscape(handler) {
    this.landmark?.addEventListener("keydown", (event) => {
      if (event.key === "Escape") {
        handler();
      }
    });
  }
  focus() {
    const active = this.landmark?.ownerDocument.activeElement ?? null;
    if (active === null || active === active.ownerDocument.body) {
      return this.dropped;
    }
    if (this.landmark?.contains(active)) {
      return "tree";
    }
    if (active === this.checkbox) {
      return "checkbox";
    }
    return active === this.button ? "button" : null;
  }
  focusButton() {
    this.button?.focus();
  }
  focusCheckbox() {
    this.checkbox?.focus();
  }
  /** A blur to nowhere of a control no longer shown is one a resize hid; a click elsewhere leaves it shown. */
  keepDropped(control, place) {
    control?.addEventListener("blur", (event) => {
      if (event.relatedTarget === null && !control.checkVisibility()) {
        this.dropped = place;
      }
    });
  }
};

// src/adapters/fetched_documents.ts
var FetchedDocuments = class {
  address;
  constructor(address) {
    this.address = address;
  }
  async load() {
    const response = await fetch(this.address);
    if (!response.ok) {
      throw new Error(`${this.address.href}: ${response.status}`);
    }
    const found2 = await response.json();
    if (!Array.isArray(found2)) {
      throw new Error(`${this.address.href}: not a list of documents`);
    }
    return found2;
  }
};

// src/adapters/fetched_tree.ts
var FetchedTree = class {
  address;
  fetcher;
  /** Chromium, through Playwright 1.63.0, refuses `fetch` called as a method of another object, so it is called on its own. */
  constructor(address, fetcher = (input, init) => fetch(input, init)) {
    this.address = address;
    this.fetcher = fetcher;
  }
  async load() {
    try {
      const response = await this.fetcher(this.address);
      return response.ok ? readTree(await response.json()) : null;
    } catch {
      return null;
    }
  }
};

// src/adapters/local_memory.ts
var LocalMemory = class {
  get(key) {
    try {
      return window.localStorage.getItem(key);
    } catch {
      return null;
    }
  }
  set(key, value) {
    try {
      window.localStorage.setItem(key, value);
    } catch {
    }
  }
};

// src/adapters/media_width.ts
var MediaWidth = class {
  query;
  constructor(query) {
    this.query = query;
  }
  narrow() {
    return this.query.matches;
  }
  onChange(handler) {
    this.query.addEventListener("change", (event) => handler(event.matches));
  }
};

// src/adapters/window_timer.ts
var WindowTimer = class {
  pending = null;
  after(milliseconds, handler) {
    this.cancel();
    this.pending = setTimeout(() => {
      this.pending = null;
      handler();
    }, milliseconds);
  }
  cancel() {
    if (this.pending !== null) {
      clearTimeout(this.pending);
      this.pending = null;
    }
  }
};

// src/policies/background_form.ts
var INLINE = "inline";
var LINKED = "linked";
function siteRoot(scriptAddress) {
  return new URL("..", scriptAddress).pathname;
}
function formKey(siteRoot2) {
  return `gherkin-glass:${siteRoot2}:backgrounds`;
}
function rememberedForm(inline) {
  return inline ? INLINE : LINKED;
}
function startForm(configured, kept) {
  if (kept === INLINE) {
    return true;
  }
  if (kept === LINKED) {
    return false;
  }
  return configured;
}

// node_modules/minisearch/dist/es/index.js
var ENTRIES = "ENTRIES";
var KEYS = "KEYS";
var VALUES = "VALUES";
var LEAF = "";
var TreeIterator = class {
  constructor(set, type) {
    const node = set._tree;
    const keys = Array.from(node.keys());
    this.set = set;
    this._type = type;
    this._path = keys.length > 0 ? [{ node, keys }] : [];
  }
  next() {
    const value = this.dive();
    this.backtrack();
    return value;
  }
  dive() {
    if (this._path.length === 0) {
      return { done: true, value: void 0 };
    }
    const { node, keys } = last$1(this._path);
    if (last$1(keys) === LEAF) {
      return { done: false, value: this.result() };
    }
    const child = node.get(last$1(keys));
    this._path.push({ node: child, keys: Array.from(child.keys()) });
    return this.dive();
  }
  backtrack() {
    if (this._path.length === 0) {
      return;
    }
    const keys = last$1(this._path).keys;
    keys.pop();
    if (keys.length > 0) {
      return;
    }
    this._path.pop();
    this.backtrack();
  }
  key() {
    return this.set._prefix + this._path.map(({ keys }) => last$1(keys)).filter((key) => key !== LEAF).join("");
  }
  value() {
    return last$1(this._path).node.get(LEAF);
  }
  result() {
    switch (this._type) {
      case VALUES:
        return this.value();
      case KEYS:
        return this.key();
      default:
        return [this.key(), this.value()];
    }
  }
  [Symbol.iterator]() {
    return this;
  }
};
var last$1 = (array) => {
  return array[array.length - 1];
};
var fuzzySearch = (node, query, maxDistance) => {
  const results = /* @__PURE__ */ new Map();
  if (query === void 0)
    return results;
  const n = query.length + 1;
  const m = n + maxDistance;
  const matrix = new Uint8Array(m * n).fill(maxDistance + 1);
  for (let j = 0; j < n; ++j)
    matrix[j] = j;
  for (let i = 1; i < m; ++i)
    matrix[i * n] = i;
  recurse(node, query, maxDistance, results, matrix, 1, n, "");
  return results;
};
var recurse = (node, query, maxDistance, results, matrix, m, n, prefix) => {
  const offset = m * n;
  key: for (const key of node.keys()) {
    if (key === LEAF) {
      const distance = matrix[offset - 1];
      if (distance <= maxDistance) {
        results.set(prefix, [node.get(key), distance]);
      }
    } else {
      let i = m;
      for (let pos = 0; pos < key.length; ++pos, ++i) {
        const char = key[pos];
        const thisRowOffset = n * i;
        const prevRowOffset = thisRowOffset - n;
        let minDistance = matrix[thisRowOffset];
        const jmin = Math.max(0, i - maxDistance - 1);
        const jmax = Math.min(n - 1, i + maxDistance);
        for (let j = jmin; j < jmax; ++j) {
          const different = char !== query[j];
          const rpl = matrix[prevRowOffset + j] + +different;
          const del = matrix[prevRowOffset + j + 1] + 1;
          const ins = matrix[thisRowOffset + j] + 1;
          const dist = matrix[thisRowOffset + j + 1] = Math.min(rpl, del, ins);
          if (dist < minDistance)
            minDistance = dist;
        }
        if (minDistance > maxDistance) {
          continue key;
        }
      }
      recurse(node.get(key), query, maxDistance, results, matrix, i, n, prefix + key);
    }
  }
};
var SearchableMap = class _SearchableMap {
  /**
   * The constructor is normally called without arguments, creating an empty
   * map. In order to create a {@link SearchableMap} from an iterable or from an
   * object, check {@link SearchableMap.from} and {@link
   * SearchableMap.fromObject}.
   *
   * The constructor arguments are for internal use, when creating derived
   * mutable views of a map at a prefix.
   */
  constructor(tree = /* @__PURE__ */ new Map(), prefix = "") {
    this._size = void 0;
    this._tree = tree;
    this._prefix = prefix;
  }
  /**
   * Creates and returns a mutable view of this {@link SearchableMap},
   * containing only entries that share the given prefix.
   *
   * ### Usage:
   *
   * ```javascript
   * let map = new SearchableMap()
   * map.set("unicorn", 1)
   * map.set("universe", 2)
   * map.set("university", 3)
   * map.set("unique", 4)
   * map.set("hello", 5)
   *
   * let uni = map.atPrefix("uni")
   * uni.get("unique") // => 4
   * uni.get("unicorn") // => 1
   * uni.get("hello") // => undefined
   *
   * let univer = map.atPrefix("univer")
   * univer.get("unique") // => undefined
   * univer.get("universe") // => 2
   * univer.get("university") // => 3
   * ```
   *
   * @param prefix  The prefix
   * @return A {@link SearchableMap} representing a mutable view of the original
   * Map at the given prefix
   */
  atPrefix(prefix) {
    if (!prefix.startsWith(this._prefix)) {
      throw new Error("Mismatched prefix");
    }
    const [node, path] = trackDown(this._tree, prefix.slice(this._prefix.length));
    if (node === void 0) {
      const [parentNode, key] = last(path);
      for (const k of parentNode.keys()) {
        if (k !== LEAF && k.startsWith(key)) {
          const node2 = /* @__PURE__ */ new Map();
          node2.set(k.slice(key.length), parentNode.get(k));
          return new _SearchableMap(node2, prefix);
        }
      }
    }
    return new _SearchableMap(node, prefix);
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/clear
   */
  clear() {
    this._size = void 0;
    this._tree.clear();
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/delete
   * @param key  Key to delete
   */
  delete(key) {
    this._size = void 0;
    return remove(this._tree, key);
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/entries
   * @return An iterator iterating through `[key, value]` entries.
   */
  entries() {
    return new TreeIterator(this, ENTRIES);
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/forEach
   * @param fn  Iteration function
   */
  forEach(fn) {
    for (const [key, value] of this) {
      fn(key, value, this);
    }
  }
  /**
   * Returns a Map of all the entries that have a key within the given edit
   * distance from the search key. The keys of the returned Map are the matching
   * keys, while the values are two-element arrays where the first element is
   * the value associated to the key, and the second is the edit distance of the
   * key to the search key.
   *
   * ### Usage:
   *
   * ```javascript
   * let map = new SearchableMap()
   * map.set('hello', 'world')
   * map.set('hell', 'yeah')
   * map.set('ciao', 'mondo')
   *
   * // Get all entries that match the key 'hallo' with a maximum edit distance of 2
   * map.fuzzyGet('hallo', 2)
   * // => Map(2) { 'hello' => ['world', 1], 'hell' => ['yeah', 2] }
   *
   * // In the example, the "hello" key has value "world" and edit distance of 1
   * // (change "e" to "a"), the key "hell" has value "yeah" and edit distance of 2
   * // (change "e" to "a", delete "o")
   * ```
   *
   * @param key  The search key
   * @param maxEditDistance  The maximum edit distance (Levenshtein)
   * @return A Map of the matching keys to their value and edit distance
   */
  fuzzyGet(key, maxEditDistance) {
    return fuzzySearch(this._tree, key, maxEditDistance);
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/get
   * @param key  Key to get
   * @return Value associated to the key, or `undefined` if the key is not
   * found.
   */
  get(key) {
    const node = lookup(this._tree, key);
    return node !== void 0 ? node.get(LEAF) : void 0;
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/has
   * @param key  Key
   * @return True if the key is in the map, false otherwise
   */
  has(key) {
    const node = lookup(this._tree, key);
    return node !== void 0 && node.has(LEAF);
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/keys
   * @return An `Iterable` iterating through keys
   */
  keys() {
    return new TreeIterator(this, KEYS);
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/set
   * @param key  Key to set
   * @param value  Value to associate to the key
   * @return The {@link SearchableMap} itself, to allow chaining
   */
  set(key, value) {
    if (typeof key !== "string") {
      throw new Error("key must be a string");
    }
    this._size = void 0;
    const node = createPath(this._tree, key);
    node.set(LEAF, value);
    return this;
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/size
   */
  get size() {
    if (this._size) {
      return this._size;
    }
    this._size = 0;
    const iter = this.entries();
    while (!iter.next().done)
      this._size += 1;
    return this._size;
  }
  /**
   * Updates the value at the given key using the provided function. The function
   * is called with the current value at the key, and its return value is used as
   * the new value to be set.
   *
   * ### Example:
   *
   * ```javascript
   * // Increment the current value by one
   * searchableMap.update('somekey', (currentValue) => currentValue == null ? 0 : currentValue + 1)
   * ```
   *
   * If the value at the given key is or will be an object, it might not require
   * re-assignment. In that case it is better to use `fetch()`, because it is
   * faster.
   *
   * @param key  The key to update
   * @param fn  The function used to compute the new value from the current one
   * @return The {@link SearchableMap} itself, to allow chaining
   */
  update(key, fn) {
    if (typeof key !== "string") {
      throw new Error("key must be a string");
    }
    this._size = void 0;
    const node = createPath(this._tree, key);
    node.set(LEAF, fn(node.get(LEAF)));
    return this;
  }
  /**
   * Fetches the value of the given key. If the value does not exist, calls the
   * given function to create a new value, which is inserted at the given key
   * and subsequently returned.
   *
   * ### Example:
   *
   * ```javascript
   * const map = searchableMap.fetch('somekey', () => new Map())
   * map.set('foo', 'bar')
   * ```
   *
   * @param key  The key to update
   * @param initial  A function that creates a new value if the key does not exist
   * @return The existing or new value at the given key
   */
  fetch(key, initial) {
    if (typeof key !== "string") {
      throw new Error("key must be a string");
    }
    this._size = void 0;
    const node = createPath(this._tree, key);
    let value = node.get(LEAF);
    if (value === void 0) {
      node.set(LEAF, value = initial());
    }
    return value;
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/values
   * @return An `Iterable` iterating through values.
   */
  values() {
    return new TreeIterator(this, VALUES);
  }
  /**
   * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/@@iterator
   */
  [Symbol.iterator]() {
    return this.entries();
  }
  /**
   * Creates a {@link SearchableMap} from an `Iterable` of entries
   *
   * @param entries  Entries to be inserted in the {@link SearchableMap}
   * @return A new {@link SearchableMap} with the given entries
   */
  static from(entries) {
    const tree = new _SearchableMap();
    for (const [key, value] of entries) {
      tree.set(key, value);
    }
    return tree;
  }
  /**
   * Creates a {@link SearchableMap} from the iterable properties of a JavaScript object
   *
   * @param object  Object of entries for the {@link SearchableMap}
   * @return A new {@link SearchableMap} with the given entries
   */
  static fromObject(object2) {
    return _SearchableMap.from(Object.entries(object2));
  }
};
var trackDown = (tree, key, path = []) => {
  if (key.length === 0 || tree == null) {
    return [tree, path];
  }
  for (const k of tree.keys()) {
    if (k !== LEAF && key.startsWith(k)) {
      path.push([tree, k]);
      return trackDown(tree.get(k), key.slice(k.length), path);
    }
  }
  path.push([tree, key]);
  return trackDown(void 0, "", path);
};
var lookup = (tree, key) => {
  if (key.length === 0 || tree == null) {
    return tree;
  }
  for (const k of tree.keys()) {
    if (k !== LEAF && key.startsWith(k)) {
      return lookup(tree.get(k), key.slice(k.length));
    }
  }
};
var createPath = (node, key) => {
  const keyLength = key.length;
  outer: for (let pos = 0; node && pos < keyLength; ) {
    for (const k of node.keys()) {
      if (k !== LEAF && key[pos] === k[0]) {
        const len = Math.min(keyLength - pos, k.length);
        let offset = 1;
        while (offset < len && key[pos + offset] === k[offset])
          ++offset;
        const child2 = node.get(k);
        if (offset === k.length) {
          node = child2;
        } else {
          const intermediate = /* @__PURE__ */ new Map();
          intermediate.set(k.slice(offset), child2);
          node.set(key.slice(pos, pos + offset), intermediate);
          node.delete(k);
          node = intermediate;
        }
        pos += offset;
        continue outer;
      }
    }
    const child = /* @__PURE__ */ new Map();
    node.set(key.slice(pos), child);
    return child;
  }
  return node;
};
var remove = (tree, key) => {
  const [node, path] = trackDown(tree, key);
  if (node === void 0) {
    return;
  }
  node.delete(LEAF);
  if (node.size === 0) {
    cleanup(path);
  } else if (node.size === 1) {
    const [key2, value] = node.entries().next().value;
    merge(path, key2, value);
  }
};
var cleanup = (path) => {
  if (path.length === 0) {
    return;
  }
  const [node, key] = last(path);
  node.delete(key);
  if (node.size === 0) {
    cleanup(path.slice(0, -1));
  } else if (node.size === 1) {
    const [key2, value] = node.entries().next().value;
    if (key2 !== LEAF) {
      merge(path.slice(0, -1), key2, value);
    }
  }
};
var merge = (path, key, value) => {
  if (path.length === 0) {
    return;
  }
  const [node, nodeKey] = last(path);
  node.set(nodeKey + key, value);
  node.delete(nodeKey);
};
var last = (array) => {
  return array[array.length - 1];
};
var OR = "or";
var AND = "and";
var AND_NOT = "and_not";
var MiniSearch = class _MiniSearch {
  /**
   * @param options  Configuration options
   *
   * ### Examples:
   *
   * ```javascript
   * // Create a search engine that indexes the 'title' and 'text' fields of your
   * // documents:
   * const miniSearch = new MiniSearch({ fields: ['title', 'text'] })
   * ```
   *
   * ### ID Field:
   *
   * ```javascript
   * // Your documents are assumed to include a unique 'id' field, but if you want
   * // to use a different field for document identification, you can set the
   * // 'idField' option:
   * const miniSearch = new MiniSearch({ idField: 'key', fields: ['title', 'text'] })
   * ```
   *
   * ### Options and defaults:
   *
   * ```javascript
   * // The full set of options (here with their default value) is:
   * const miniSearch = new MiniSearch({
   *   // idField: field that uniquely identifies a document
   *   idField: 'id',
   *
   *   // extractField: function used to get the value of a field in a document.
   *   // By default, it assumes the document is a flat object with field names as
   *   // property keys and field values as string property values, but custom logic
   *   // can be implemented by setting this option to a custom extractor function.
   *   extractField: (document, fieldName) => document[fieldName],
   *
   *   // tokenize: function used to split fields into individual terms. By
   *   // default, it is also used to tokenize search queries, unless a specific
   *   // `tokenize` search option is supplied. When tokenizing an indexed field,
   *   // the field name is passed as the second argument.
   *   tokenize: (string, _fieldName) => string.split(SPACE_OR_PUNCTUATION),
   *
   *   // processTerm: function used to process each tokenized term before
   *   // indexing. It can be used for stemming and normalization. Return a falsy
   *   // value in order to discard a term. By default, it is also used to process
   *   // search queries, unless a specific `processTerm` option is supplied as a
   *   // search option. When processing a term from a indexed field, the field
   *   // name is passed as the second argument.
   *   processTerm: (term, _fieldName) => term.toLowerCase(),
   *
   *   // searchOptions: default search options, see the `search` method for
   *   // details
   *   searchOptions: undefined,
   *
   *   // fields: document fields to be indexed. Mandatory, but not set by default
   *   fields: undefined
   *
   *   // storeFields: document fields to be stored and returned as part of the
   *   // search results.
   *   storeFields: []
   * })
   * ```
   */
  constructor(options) {
    if ((options === null || options === void 0 ? void 0 : options.fields) == null) {
      throw new Error('MiniSearch: option "fields" must be provided');
    }
    const autoVacuum = options.autoVacuum == null || options.autoVacuum === true ? defaultAutoVacuumOptions : options.autoVacuum;
    this._options = {
      ...defaultOptions,
      ...options,
      autoVacuum,
      searchOptions: { ...defaultSearchOptions, ...options.searchOptions || {} },
      autoSuggestOptions: { ...defaultAutoSuggestOptions, ...options.autoSuggestOptions || {} }
    };
    this._index = new SearchableMap();
    this._documentCount = 0;
    this._documentIds = /* @__PURE__ */ new Map();
    this._idToShortId = /* @__PURE__ */ new Map();
    this._fieldIds = {};
    this._fieldLength = /* @__PURE__ */ new Map();
    this._avgFieldLength = [];
    this._nextId = 0;
    this._storedFields = /* @__PURE__ */ new Map();
    this._dirtCount = 0;
    this._currentVacuum = null;
    this._enqueuedVacuum = null;
    this._enqueuedVacuumConditions = defaultVacuumConditions;
    this.addFields(this._options.fields);
  }
  /**
   * Adds a document to the index
   *
   * @param document  The document to be indexed
   */
  add(document2) {
    const { extractField, stringifyField, tokenize, processTerm, fields, idField } = this._options;
    const id = extractField(document2, idField);
    if (id == null) {
      throw new Error(`MiniSearch: document does not have ID field "${idField}"`);
    }
    if (this._idToShortId.has(id)) {
      throw new Error(`MiniSearch: duplicate ID ${id}`);
    }
    const shortDocumentId = this.addDocumentId(id);
    this.saveStoredFields(shortDocumentId, document2);
    for (const field of fields) {
      const fieldValue = extractField(document2, field);
      if (fieldValue == null)
        continue;
      const tokens = tokenize(stringifyField(fieldValue, field), field);
      const fieldId = this._fieldIds[field];
      const uniqueTerms = new Set(tokens).size;
      this.addFieldLength(shortDocumentId, fieldId, this._documentCount - 1, uniqueTerms);
      for (const term of tokens) {
        const processedTerm = processTerm(term, field);
        if (Array.isArray(processedTerm)) {
          for (const t of processedTerm) {
            this.addTerm(fieldId, shortDocumentId, t);
          }
        } else if (processedTerm) {
          this.addTerm(fieldId, shortDocumentId, processedTerm);
        }
      }
    }
  }
  /**
   * Adds all the given documents to the index
   *
   * @param documents  An array of documents to be indexed
   */
  addAll(documents) {
    for (const document2 of documents)
      this.add(document2);
  }
  /**
   * Adds all the given documents to the index asynchronously.
   *
   * Returns a promise that resolves (to `undefined`) when the indexing is done.
   * This method is useful when index many documents, to avoid blocking the main
   * thread. The indexing is performed asynchronously and in chunks.
   *
   * @param documents  An array of documents to be indexed
   * @param options  Configuration options
   * @return A promise resolving to `undefined` when the indexing is done
   */
  addAllAsync(documents, options = {}) {
    const { chunkSize = 10 } = options;
    const acc = { chunk: [], promise: Promise.resolve() };
    const { chunk, promise } = documents.reduce(({ chunk: chunk2, promise: promise2 }, document2, i) => {
      chunk2.push(document2);
      if ((i + 1) % chunkSize === 0) {
        return {
          chunk: [],
          promise: promise2.then(() => new Promise((resolve) => setTimeout(resolve, 0))).then(() => this.addAll(chunk2))
        };
      } else {
        return { chunk: chunk2, promise: promise2 };
      }
    }, acc);
    return promise.then(() => this.addAll(chunk));
  }
  /**
   * Removes the given document from the index.
   *
   * The document to remove must NOT have changed between indexing and removal,
   * otherwise the index will be corrupted.
   *
   * This method requires passing the full document to be removed (not just the
   * ID), and immediately removes the document from the inverted index, allowing
   * memory to be released. A convenient alternative is {@link
   * MiniSearch#discard}, which needs only the document ID, and has the same
   * visible effect, but delays cleaning up the index until the next vacuuming.
   *
   * @param document  The document to be removed
   */
  remove(document2) {
    const { tokenize, processTerm, extractField, stringifyField, fields, idField } = this._options;
    const id = extractField(document2, idField);
    if (id == null) {
      throw new Error(`MiniSearch: document does not have ID field "${idField}"`);
    }
    const shortId = this._idToShortId.get(id);
    if (shortId == null) {
      throw new Error(`MiniSearch: cannot remove document with ID ${id}: it is not in the index`);
    }
    for (const field of fields) {
      const fieldValue = extractField(document2, field);
      if (fieldValue == null)
        continue;
      const tokens = tokenize(stringifyField(fieldValue, field), field);
      const fieldId = this._fieldIds[field];
      const uniqueTerms = new Set(tokens).size;
      this.removeFieldLength(shortId, fieldId, this._documentCount, uniqueTerms);
      for (const term of tokens) {
        const processedTerm = processTerm(term, field);
        if (Array.isArray(processedTerm)) {
          for (const t of processedTerm) {
            this.removeTerm(fieldId, shortId, t);
          }
        } else if (processedTerm) {
          this.removeTerm(fieldId, shortId, processedTerm);
        }
      }
    }
    this._storedFields.delete(shortId);
    this._documentIds.delete(shortId);
    this._idToShortId.delete(id);
    this._fieldLength.delete(shortId);
    this._documentCount -= 1;
  }
  /**
   * Removes all the given documents from the index. If called with no arguments,
   * it removes _all_ documents from the index.
   *
   * @param documents  The documents to be removed. If this argument is omitted,
   * all documents are removed. Note that, for removing all documents, it is
   * more efficient to call this method with no arguments than to pass all
   * documents.
   */
  removeAll(documents) {
    if (documents) {
      for (const document2 of documents)
        this.remove(document2);
    } else if (arguments.length > 0) {
      throw new Error("Expected documents to be present. Omit the argument to remove all documents.");
    } else {
      this._index = new SearchableMap();
      this._documentCount = 0;
      this._documentIds = /* @__PURE__ */ new Map();
      this._idToShortId = /* @__PURE__ */ new Map();
      this._fieldLength = /* @__PURE__ */ new Map();
      this._avgFieldLength = [];
      this._storedFields = /* @__PURE__ */ new Map();
      this._nextId = 0;
    }
  }
  /**
   * Discards the document with the given ID, so it won't appear in search results
   *
   * It has the same visible effect of {@link MiniSearch.remove} (both cause the
   * document to stop appearing in searches), but a different effect on the
   * internal data structures:
   *
   *   - {@link MiniSearch#remove} requires passing the full document to be
   *   removed as argument, and removes it from the inverted index immediately.
   *
   *   - {@link MiniSearch#discard} instead only needs the document ID, and
   *   works by marking the current version of the document as discarded, so it
   *   is immediately ignored by searches. This is faster and more convenient
   *   than {@link MiniSearch#remove}, but the index is not immediately
   *   modified. To take care of that, vacuuming is performed after a certain
   *   number of documents are discarded, cleaning up the index and allowing
   *   memory to be released.
   *
   * After discarding a document, it is possible to re-add a new version, and
   * only the new version will appear in searches. In other words, discarding
   * and re-adding a document works exactly like removing and re-adding it. The
   * {@link MiniSearch.replace} method can also be used to replace a document
   * with a new version.
   *
   * #### Details about vacuuming
   *
   * Repetite calls to this method would leave obsolete document references in
   * the index, invisible to searches. Two mechanisms take care of cleaning up:
   * clean up during search, and vacuuming.
   *
   *   - Upon search, whenever a discarded ID is found (and ignored for the
   *   results), references to the discarded document are removed from the
   *   inverted index entries for the search terms. This ensures that subsequent
   *   searches for the same terms do not need to skip these obsolete references
   *   again.
   *
   *   - In addition, vacuuming is performed automatically by default (see the
   *   `autoVacuum` field in {@link Options}) after a certain number of
   *   documents are discarded. Vacuuming traverses all terms in the index,
   *   cleaning up all references to discarded documents. Vacuuming can also be
   *   triggered manually by calling {@link MiniSearch#vacuum}.
   *
   * @param id  The ID of the document to be discarded
   */
  discard(id) {
    const shortId = this._idToShortId.get(id);
    if (shortId == null) {
      throw new Error(`MiniSearch: cannot discard document with ID ${id}: it is not in the index`);
    }
    this._idToShortId.delete(id);
    this._documentIds.delete(shortId);
    this._storedFields.delete(shortId);
    (this._fieldLength.get(shortId) || []).forEach((fieldLength, fieldId) => {
      this.removeFieldLength(shortId, fieldId, this._documentCount, fieldLength);
    });
    this._fieldLength.delete(shortId);
    this._documentCount -= 1;
    this._dirtCount += 1;
    this.maybeAutoVacuum();
  }
  maybeAutoVacuum() {
    if (this._options.autoVacuum === false) {
      return;
    }
    const { minDirtFactor, minDirtCount, batchSize, batchWait } = this._options.autoVacuum;
    this.conditionalVacuum({ batchSize, batchWait }, { minDirtCount, minDirtFactor });
  }
  /**
   * Discards the documents with the given IDs, so they won't appear in search
   * results
   *
   * It is equivalent to calling {@link MiniSearch#discard} for all the given
   * IDs, but with the optimization of triggering at most one automatic
   * vacuuming at the end.
   *
   * Note: to remove all documents from the index, it is faster and more
   * convenient to call {@link MiniSearch.removeAll} with no argument, instead
   * of passing all IDs to this method.
   */
  discardAll(ids) {
    const autoVacuum = this._options.autoVacuum;
    try {
      this._options.autoVacuum = false;
      for (const id of ids) {
        this.discard(id);
      }
    } finally {
      this._options.autoVacuum = autoVacuum;
    }
    this.maybeAutoVacuum();
  }
  /**
   * It replaces an existing document with the given updated version
   *
   * It works by discarding the current version and adding the updated one, so
   * it is functionally equivalent to calling {@link MiniSearch#discard}
   * followed by {@link MiniSearch#add}. The ID of the updated document should
   * be the same as the original one.
   *
   * Since it uses {@link MiniSearch#discard} internally, this method relies on
   * vacuuming to clean up obsolete document references from the index, allowing
   * memory to be released (see {@link MiniSearch#discard}).
   *
   * @param updatedDocument  The updated document to replace the old version
   * with
   */
  replace(updatedDocument) {
    const { idField, extractField } = this._options;
    const id = extractField(updatedDocument, idField);
    this.discard(id);
    this.add(updatedDocument);
  }
  /**
   * Triggers a manual vacuuming, cleaning up references to discarded documents
   * from the inverted index
   *
   * Vacuuming is only useful for applications that use the {@link
   * MiniSearch#discard} or {@link MiniSearch#replace} methods.
   *
   * By default, vacuuming is performed automatically when needed (controlled by
   * the `autoVacuum` field in {@link Options}), so there is usually no need to
   * call this method, unless one wants to make sure to perform vacuuming at a
   * specific moment.
   *
   * Vacuuming traverses all terms in the inverted index in batches, and cleans
   * up references to discarded documents from the posting list, allowing memory
   * to be released.
   *
   * The method takes an optional object as argument with the following keys:
   *
   *   - `batchSize`: the size of each batch (1000 by default)
   *
   *   - `batchWait`: the number of milliseconds to wait between batches (10 by
   *   default)
   *
   * On large indexes, vacuuming could have a non-negligible cost: batching
   * avoids blocking the thread for long, diluting this cost so that it is not
   * negatively affecting the application. Nonetheless, this method should only
   * be called when necessary, and relying on automatic vacuuming is usually
   * better.
   *
   * It returns a promise that resolves (to undefined) when the clean up is
   * completed. If vacuuming is already ongoing at the time this method is
   * called, a new one is enqueued immediately after the ongoing one, and a
   * corresponding promise is returned. However, no more than one vacuuming is
   * enqueued on top of the ongoing one, even if this method is called more
   * times (enqueuing multiple ones would be useless).
   *
   * @param options  Configuration options for the batch size and delay. See
   * {@link VacuumOptions}.
   */
  vacuum(options = {}) {
    return this.conditionalVacuum(options);
  }
  conditionalVacuum(options, conditions) {
    if (this._currentVacuum) {
      this._enqueuedVacuumConditions = this._enqueuedVacuumConditions && conditions;
      if (this._enqueuedVacuum != null) {
        return this._enqueuedVacuum;
      }
      this._enqueuedVacuum = this._currentVacuum.then(() => {
        const conditions2 = this._enqueuedVacuumConditions;
        this._enqueuedVacuumConditions = defaultVacuumConditions;
        return this.performVacuuming(options, conditions2);
      });
      return this._enqueuedVacuum;
    }
    if (this.vacuumConditionsMet(conditions) === false) {
      return Promise.resolve();
    }
    this._currentVacuum = this.performVacuuming(options);
    return this._currentVacuum;
  }
  async performVacuuming(options, conditions) {
    const initialDirtCount = this._dirtCount;
    if (this.vacuumConditionsMet(conditions)) {
      const batchSize = options.batchSize || defaultVacuumOptions.batchSize;
      const batchWait = options.batchWait || defaultVacuumOptions.batchWait;
      let i = 1;
      for (const [term, fieldsData] of this._index) {
        for (const [fieldId, fieldIndex] of fieldsData) {
          for (const [shortId] of fieldIndex) {
            if (this._documentIds.has(shortId)) {
              continue;
            }
            if (fieldIndex.size <= 1) {
              fieldsData.delete(fieldId);
            } else {
              fieldIndex.delete(shortId);
            }
          }
        }
        if (this._index.get(term).size === 0) {
          this._index.delete(term);
        }
        if (i % batchSize === 0) {
          await new Promise((resolve) => setTimeout(resolve, batchWait));
        }
        i += 1;
      }
      this._dirtCount -= initialDirtCount;
    }
    await null;
    this._currentVacuum = this._enqueuedVacuum;
    this._enqueuedVacuum = null;
  }
  vacuumConditionsMet(conditions) {
    if (conditions == null) {
      return true;
    }
    let { minDirtCount, minDirtFactor } = conditions;
    minDirtCount = minDirtCount || defaultAutoVacuumOptions.minDirtCount;
    minDirtFactor = minDirtFactor || defaultAutoVacuumOptions.minDirtFactor;
    return this.dirtCount >= minDirtCount && this.dirtFactor >= minDirtFactor;
  }
  /**
   * Is `true` if a vacuuming operation is ongoing, `false` otherwise
   */
  get isVacuuming() {
    return this._currentVacuum != null;
  }
  /**
   * The number of documents discarded since the most recent vacuuming
   */
  get dirtCount() {
    return this._dirtCount;
  }
  /**
   * A number between 0 and 1 giving an indication about the proportion of
   * documents that are discarded, and can therefore be cleaned up by vacuuming.
   * A value close to 0 means that the index is relatively clean, while a higher
   * value means that the index is relatively dirty, and vacuuming could release
   * memory.
   */
  get dirtFactor() {
    return this._dirtCount / (1 + this._documentCount + this._dirtCount);
  }
  /**
   * Returns `true` if a document with the given ID is present in the index and
   * available for search, `false` otherwise
   *
   * @param id  The document ID
   */
  has(id) {
    return this._idToShortId.has(id);
  }
  /**
   * Returns the stored fields (as configured in the `storeFields` constructor
   * option) for the given document ID. Returns `undefined` if the document is
   * not present in the index.
   *
   * @param id  The document ID
   */
  getStoredFields(id) {
    const shortId = this._idToShortId.get(id);
    if (shortId == null) {
      return void 0;
    }
    return this._storedFields.get(shortId);
  }
  /**
   * Search for documents matching the given search query.
   *
   * The result is a list of scored document IDs matching the query, sorted by
   * descending score, and each including data about which terms were matched and
   * in which fields.
   *
   * ### Basic usage:
   *
   * ```javascript
   * // Search for "zen art motorcycle" with default options: terms have to match
   * // exactly, and individual terms are joined with OR
   * miniSearch.search('zen art motorcycle')
   * // => [ { id: 2, score: 2.77258, match: { ... } }, { id: 4, score: 1.38629, match: { ... } } ]
   * ```
   *
   * ### Restrict search to specific fields:
   *
   * ```javascript
   * // Search only in the 'title' field
   * miniSearch.search('zen', { fields: ['title'] })
   * ```
   *
   * ### Field boosting:
   *
   * ```javascript
   * // Boost a field
   * miniSearch.search('zen', { boost: { title: 2 } })
   * ```
   *
   * ### Prefix search:
   *
   * ```javascript
   * // Search for "moto" with prefix search (it will match documents
   * // containing terms that start with "moto" or "neuro")
   * miniSearch.search('moto neuro', { prefix: true })
   * ```
   *
   * ### Fuzzy search:
   *
   * ```javascript
   * // Search for "ismael" with fuzzy search (it will match documents containing
   * // terms similar to "ismael", with a maximum edit distance of 0.2 term.length
   * // (rounded to nearest integer)
   * miniSearch.search('ismael', { fuzzy: 0.2 })
   * ```
   *
   * ### Combining strategies:
   *
   * ```javascript
   * // Mix of exact match, prefix search, and fuzzy search
   * miniSearch.search('ismael mob', {
   *  prefix: true,
   *  fuzzy: 0.2
   * })
   * ```
   *
   * ### Advanced prefix and fuzzy search:
   *
   * ```javascript
   * // Perform fuzzy and prefix search depending on the search term. Here
   * // performing prefix and fuzzy search only on terms longer than 3 characters
   * miniSearch.search('ismael mob', {
   *  prefix: term => term.length > 3
   *  fuzzy: term => term.length > 3 ? 0.2 : null
   * })
   * ```
   *
   * ### Combine with AND:
   *
   * ```javascript
   * // Combine search terms with AND (to match only documents that contain both
   * // "motorcycle" and "art")
   * miniSearch.search('motorcycle art', { combineWith: 'AND' })
   * ```
   *
   * ### Combine with AND_NOT:
   *
   * There is also an AND_NOT combinator, that finds documents that match the
   * first term, but do not match any of the other terms. This combinator is
   * rarely useful with simple queries, and is meant to be used with advanced
   * query combinations (see later for more details).
   *
   * ### Filtering results:
   *
   * ```javascript
   * // Filter only results in the 'fiction' category (assuming that 'category'
   * // is a stored field)
   * miniSearch.search('motorcycle art', {
   *   filter: (result) => result.category === 'fiction'
   * })
   * ```
   *
   * ### Wildcard query
   *
   * Searching for an empty string (assuming the default tokenizer) returns no
   * results. Sometimes though, one needs to match all documents, like in a
   * "wildcard" search. This is possible by passing the special value
   * {@link MiniSearch.wildcard} as the query:
   *
   * ```javascript
   * // Return search results for all documents
   * miniSearch.search(MiniSearch.wildcard)
   * ```
   *
   * Note that search options such as `filter` and `boostDocument` are still
   * applied, influencing which results are returned, and their order:
   *
   * ```javascript
   * // Return search results for all documents in the 'fiction' category
   * miniSearch.search(MiniSearch.wildcard, {
   *   filter: (result) => result.category === 'fiction'
   * })
   * ```
   *
   * ### Advanced combination of queries:
   *
   * It is possible to combine different subqueries with OR, AND, and AND_NOT,
   * and even with different search options, by passing a query expression
   * tree object as the first argument, instead of a string.
   *
   * ```javascript
   * // Search for documents that contain "zen" and ("motorcycle" or "archery")
   * miniSearch.search({
   *   combineWith: 'AND',
   *   queries: [
   *     'zen',
   *     {
   *       combineWith: 'OR',
   *       queries: ['motorcycle', 'archery']
   *     }
   *   ]
   * })
   *
   * // Search for documents that contain ("apple" or "pear") but not "juice" and
   * // not "tree"
   * miniSearch.search({
   *   combineWith: 'AND_NOT',
   *   queries: [
   *     {
   *       combineWith: 'OR',
   *       queries: ['apple', 'pear']
   *     },
   *     'juice',
   *     'tree'
   *   ]
   * })
   * ```
   *
   * Each node in the expression tree can be either a string, or an object that
   * supports all {@link SearchOptions} fields, plus a `queries` array field for
   * subqueries.
   *
   * Note that, while this can become complicated to do by hand for complex or
   * deeply nested queries, it provides a formalized expression tree API for
   * external libraries that implement a parser for custom query languages.
   *
   * @param query  Search query
   * @param searchOptions  Search options. Each option, if not given, defaults to the corresponding value of `searchOptions` given to the constructor, or to the library default.
   */
  search(query, searchOptions = {}) {
    const { searchOptions: globalSearchOptions } = this._options;
    const searchOptionsWithDefaults = { ...globalSearchOptions, ...searchOptions };
    const rawResults = this.executeQuery(query, searchOptions);
    const results = [];
    for (const [docId, { score, terms, match: match2 }] of rawResults) {
      const quality = terms.length || 1;
      const result2 = {
        id: this._documentIds.get(docId),
        score: score * quality,
        terms: Object.keys(match2),
        queryTerms: terms,
        match: match2
      };
      Object.assign(result2, this._storedFields.get(docId));
      if (searchOptionsWithDefaults.filter == null || searchOptionsWithDefaults.filter(result2)) {
        results.push(result2);
      }
    }
    if (query === _MiniSearch.wildcard && searchOptionsWithDefaults.boostDocument == null) {
      return results;
    }
    results.sort(byScore);
    return results;
  }
  /**
   * Provide suggestions for the given search query
   *
   * The result is a list of suggested modified search queries, derived from the
   * given search query, each with a relevance score, sorted by descending score.
   *
   * By default, it uses the same options used for search, except that by
   * default it performs prefix search on the last term of the query, and
   * combine terms with `'AND'` (requiring all query terms to match). Custom
   * options can be passed as a second argument. Defaults can be changed upon
   * calling the {@link MiniSearch} constructor, by passing a
   * `autoSuggestOptions` option.
   *
   * ### Basic usage:
   *
   * ```javascript
   * // Get suggestions for 'neuro':
   * miniSearch.autoSuggest('neuro')
   * // => [ { suggestion: 'neuromancer', terms: [ 'neuromancer' ], score: 0.46240 } ]
   * ```
   *
   * ### Multiple words:
   *
   * ```javascript
   * // Get suggestions for 'zen ar':
   * miniSearch.autoSuggest('zen ar')
   * // => [
   * //  { suggestion: 'zen archery art', terms: [ 'zen', 'archery', 'art' ], score: 1.73332 },
   * //  { suggestion: 'zen art', terms: [ 'zen', 'art' ], score: 1.21313 }
   * // ]
   * ```
   *
   * ### Fuzzy suggestions:
   *
   * ```javascript
   * // Correct spelling mistakes using fuzzy search:
   * miniSearch.autoSuggest('neromancer', { fuzzy: 0.2 })
   * // => [ { suggestion: 'neuromancer', terms: [ 'neuromancer' ], score: 1.03998 } ]
   * ```
   *
   * ### Filtering:
   *
   * ```javascript
   * // Get suggestions for 'zen ar', but only within the 'fiction' category
   * // (assuming that 'category' is a stored field):
   * miniSearch.autoSuggest('zen ar', {
   *   filter: (result) => result.category === 'fiction'
   * })
   * // => [
   * //  { suggestion: 'zen archery art', terms: [ 'zen', 'archery', 'art' ], score: 1.73332 },
   * //  { suggestion: 'zen art', terms: [ 'zen', 'art' ], score: 1.21313 }
   * // ]
   * ```
   *
   * @param queryString  Query string to be expanded into suggestions
   * @param options  Search options. The supported options and default values
   * are the same as for the {@link MiniSearch#search} method, except that by
   * default prefix search is performed on the last term in the query, and terms
   * are combined with `'AND'`.
   * @return  A sorted array of suggestions sorted by relevance score.
   */
  autoSuggest(queryString, options = {}) {
    options = { ...this._options.autoSuggestOptions, ...options };
    const suggestions = /* @__PURE__ */ new Map();
    for (const { score, terms } of this.search(queryString, options)) {
      const phrase = terms.join(" ");
      const suggestion = suggestions.get(phrase);
      if (suggestion != null) {
        suggestion.score += score;
        suggestion.count += 1;
      } else {
        suggestions.set(phrase, { score, terms, count: 1 });
      }
    }
    const results = [];
    for (const [suggestion, { score, terms, count }] of suggestions) {
      results.push({ suggestion, terms, score: score / count });
    }
    results.sort(byScore);
    return results;
  }
  /**
   * Total number of documents available to search
   */
  get documentCount() {
    return this._documentCount;
  }
  /**
   * Number of terms in the index
   */
  get termCount() {
    return this._index.size;
  }
  /**
   * Deserializes a JSON index (serialized with `JSON.stringify(miniSearch)`)
   * and instantiates a MiniSearch instance. It should be given the same options
   * originally used when serializing the index.
   *
   * ### Usage:
   *
   * ```javascript
   * // If the index was serialized with:
   * let miniSearch = new MiniSearch({ fields: ['title', 'text'] })
   * miniSearch.addAll(documents)
   *
   * const json = JSON.stringify(miniSearch)
   * // It can later be deserialized like this:
   * miniSearch = MiniSearch.loadJSON(json, { fields: ['title', 'text'] })
   * ```
   *
   * @param json  JSON-serialized index
   * @param options  configuration options, same as the constructor
   * @return An instance of MiniSearch deserialized from the given JSON.
   */
  static loadJSON(json, options) {
    if (options == null) {
      throw new Error("MiniSearch: loadJSON should be given the same options used when serializing the index");
    }
    return this.loadJS(JSON.parse(json), options);
  }
  /**
   * Async equivalent of {@link MiniSearch.loadJSON}
   *
   * This function is an alternative to {@link MiniSearch.loadJSON} that returns
   * a promise, and loads the index in batches, leaving pauses between them to avoid
   * blocking the main thread. It tends to be slower than the synchronous
   * version, but does not block the main thread, so it can be a better choice
   * when deserializing very large indexes.
   *
   * @param json  JSON-serialized index
   * @param options  configuration options, same as the constructor
   * @return A Promise that will resolve to an instance of MiniSearch deserialized from the given JSON.
   */
  static async loadJSONAsync(json, options) {
    if (options == null) {
      throw new Error("MiniSearch: loadJSON should be given the same options used when serializing the index");
    }
    return this.loadJSAsync(JSON.parse(json), options);
  }
  /**
   * Returns the default value of an option. It will throw an error if no option
   * with the given name exists.
   *
   * @param optionName  Name of the option
   * @return The default value of the given option
   *
   * ### Usage:
   *
   * ```javascript
   * // Get default tokenizer
   * MiniSearch.getDefault('tokenize')
   *
   * // Get default term processor
   * MiniSearch.getDefault('processTerm')
   *
   * // Unknown options will throw an error
   * MiniSearch.getDefault('notExisting')
   * // => throws 'MiniSearch: unknown option "notExisting"'
   * ```
   */
  static getDefault(optionName) {
    if (defaultOptions.hasOwnProperty(optionName)) {
      return getOwnProperty(defaultOptions, optionName);
    } else {
      throw new Error(`MiniSearch: unknown option "${optionName}"`);
    }
  }
  /**
   * @ignore
   */
  static loadJS(js, options) {
    const { index, documentIds, fieldLength, storedFields, serializationVersion } = js;
    const miniSearch = this.instantiateMiniSearch(js, options);
    miniSearch._documentIds = objectToNumericMap(documentIds);
    miniSearch._fieldLength = objectToNumericMap(fieldLength);
    miniSearch._storedFields = objectToNumericMap(storedFields);
    for (const [shortId, id] of miniSearch._documentIds) {
      miniSearch._idToShortId.set(id, shortId);
    }
    for (const [term, data] of index) {
      const dataMap = /* @__PURE__ */ new Map();
      for (const fieldId of Object.keys(data)) {
        let indexEntry = data[fieldId];
        if (serializationVersion === 1) {
          indexEntry = indexEntry.ds;
        }
        dataMap.set(parseInt(fieldId, 10), objectToNumericMap(indexEntry));
      }
      miniSearch._index.set(term, dataMap);
    }
    return miniSearch;
  }
  /**
   * @ignore
   */
  static async loadJSAsync(js, options) {
    const { index, documentIds, fieldLength, storedFields, serializationVersion } = js;
    const miniSearch = this.instantiateMiniSearch(js, options);
    miniSearch._documentIds = await objectToNumericMapAsync(documentIds);
    miniSearch._fieldLength = await objectToNumericMapAsync(fieldLength);
    miniSearch._storedFields = await objectToNumericMapAsync(storedFields);
    for (const [shortId, id] of miniSearch._documentIds) {
      miniSearch._idToShortId.set(id, shortId);
    }
    let count = 0;
    for (const [term, data] of index) {
      const dataMap = /* @__PURE__ */ new Map();
      for (const fieldId of Object.keys(data)) {
        let indexEntry = data[fieldId];
        if (serializationVersion === 1) {
          indexEntry = indexEntry.ds;
        }
        dataMap.set(parseInt(fieldId, 10), await objectToNumericMapAsync(indexEntry));
      }
      if (++count % 1e3 === 0)
        await wait(0);
      miniSearch._index.set(term, dataMap);
    }
    return miniSearch;
  }
  /**
   * @ignore
   */
  static instantiateMiniSearch(js, options) {
    const { documentCount, nextId, fieldIds, averageFieldLength, dirtCount, serializationVersion } = js;
    if (serializationVersion !== 1 && serializationVersion !== 2) {
      throw new Error("MiniSearch: cannot deserialize an index created with an incompatible version");
    }
    const miniSearch = new _MiniSearch(options);
    miniSearch._documentCount = documentCount;
    miniSearch._nextId = nextId;
    miniSearch._idToShortId = /* @__PURE__ */ new Map();
    miniSearch._fieldIds = fieldIds;
    miniSearch._avgFieldLength = averageFieldLength;
    miniSearch._dirtCount = dirtCount || 0;
    miniSearch._index = new SearchableMap();
    return miniSearch;
  }
  /**
   * @ignore
   */
  executeQuery(query, searchOptions = {}) {
    if (query === _MiniSearch.wildcard) {
      return this.executeWildcardQuery(searchOptions);
    }
    if (typeof query !== "string") {
      const options2 = { ...searchOptions, ...query, queries: void 0 };
      const results2 = query.queries.map((subquery) => this.executeQuery(subquery, options2));
      return this.combineResults(results2, options2.combineWith);
    }
    const { tokenize, processTerm, searchOptions: globalSearchOptions } = this._options;
    const options = { tokenize, processTerm, ...globalSearchOptions, ...searchOptions };
    const { tokenize: searchTokenize, processTerm: searchProcessTerm } = options;
    const terms = searchTokenize(query).flatMap((term) => searchProcessTerm(term)).filter((term) => !!term);
    const queries = terms.map(termToQuerySpec(options));
    const results = queries.map((query2) => this.executeQuerySpec(query2, options));
    return this.combineResults(results, options.combineWith);
  }
  /**
   * @ignore
   */
  executeQuerySpec(query, searchOptions) {
    const options = { ...this._options.searchOptions, ...searchOptions };
    const boosts = (options.fields || this._options.fields).reduce((boosts2, field) => ({ ...boosts2, [field]: getOwnProperty(options.boost, field) || 1 }), {});
    const { boostDocument, weights, maxFuzzy, bm25: bm25params } = options;
    const { fuzzy: fuzzyWeight, prefix: prefixWeight } = { ...defaultSearchOptions.weights, ...weights };
    const data = this._index.get(query.term);
    const results = this.termResults(query.term, query.term, 1, query.termBoost, data, boosts, boostDocument, bm25params);
    let prefixMatches;
    let fuzzyMatches;
    if (query.prefix) {
      prefixMatches = this._index.atPrefix(query.term);
    }
    if (query.fuzzy) {
      const fuzzy = query.fuzzy === true ? 0.2 : query.fuzzy;
      const maxDistance = fuzzy < 1 ? Math.min(maxFuzzy, Math.round(query.term.length * fuzzy)) : fuzzy;
      if (maxDistance)
        fuzzyMatches = this._index.fuzzyGet(query.term, maxDistance);
    }
    if (prefixMatches) {
      for (const [term, data2] of prefixMatches) {
        const distance = term.length - query.term.length;
        if (!distance) {
          continue;
        }
        fuzzyMatches === null || fuzzyMatches === void 0 ? void 0 : fuzzyMatches.delete(term);
        const weight = prefixWeight * term.length / (term.length + 0.3 * distance);
        this.termResults(query.term, term, weight, query.termBoost, data2, boosts, boostDocument, bm25params, results);
      }
    }
    if (fuzzyMatches) {
      for (const term of fuzzyMatches.keys()) {
        const [data2, distance] = fuzzyMatches.get(term);
        if (!distance) {
          continue;
        }
        const weight = fuzzyWeight * term.length / (term.length + distance);
        this.termResults(query.term, term, weight, query.termBoost, data2, boosts, boostDocument, bm25params, results);
      }
    }
    return results;
  }
  /**
   * @ignore
   */
  executeWildcardQuery(searchOptions) {
    const results = /* @__PURE__ */ new Map();
    const options = { ...this._options.searchOptions, ...searchOptions };
    for (const [shortId, id] of this._documentIds) {
      const score = options.boostDocument ? options.boostDocument(id, "", this._storedFields.get(shortId)) : 1;
      results.set(shortId, {
        score,
        terms: [],
        match: {}
      });
    }
    return results;
  }
  /**
   * @ignore
   */
  combineResults(results, combineWith = OR) {
    if (results.length === 0) {
      return /* @__PURE__ */ new Map();
    }
    const operator = combineWith.toLowerCase();
    const combinator = combinators[operator];
    if (!combinator) {
      throw new Error(`Invalid combination operator: ${combineWith}`);
    }
    return results.reduce(combinator) || /* @__PURE__ */ new Map();
  }
  /**
   * Allows serialization of the index to JSON, to possibly store it and later
   * deserialize it with {@link MiniSearch.loadJSON}.
   *
   * Normally one does not directly call this method, but rather call the
   * standard JavaScript `JSON.stringify()` passing the {@link MiniSearch}
   * instance, and JavaScript will internally call this method. Upon
   * deserialization, one must pass to {@link MiniSearch.loadJSON} the same
   * options used to create the original instance that was serialized.
   *
   * ### Usage:
   *
   * ```javascript
   * // Serialize the index:
   * let miniSearch = new MiniSearch({ fields: ['title', 'text'] })
   * miniSearch.addAll(documents)
   * const json = JSON.stringify(miniSearch)
   *
   * // Later, to deserialize it:
   * miniSearch = MiniSearch.loadJSON(json, { fields: ['title', 'text'] })
   * ```
   *
   * @return A plain-object serializable representation of the search index.
   */
  toJSON() {
    const index = [];
    for (const [term, fieldIndex] of this._index) {
      const data = {};
      for (const [fieldId, freqs] of fieldIndex) {
        data[fieldId] = Object.fromEntries(freqs);
      }
      index.push([term, data]);
    }
    return {
      documentCount: this._documentCount,
      nextId: this._nextId,
      documentIds: Object.fromEntries(this._documentIds),
      fieldIds: this._fieldIds,
      fieldLength: Object.fromEntries(this._fieldLength),
      averageFieldLength: this._avgFieldLength,
      storedFields: Object.fromEntries(this._storedFields),
      dirtCount: this._dirtCount,
      index,
      serializationVersion: 2
    };
  }
  /**
   * @ignore
   */
  termResults(sourceTerm, derivedTerm, termWeight, termBoost, fieldTermData, fieldBoosts, boostDocumentFn, bm25params, results = /* @__PURE__ */ new Map()) {
    if (fieldTermData == null)
      return results;
    for (const field of Object.keys(fieldBoosts)) {
      const fieldBoost = fieldBoosts[field];
      const fieldId = this._fieldIds[field];
      const fieldTermFreqs = fieldTermData.get(fieldId);
      if (fieldTermFreqs == null)
        continue;
      let matchingFields = fieldTermFreqs.size;
      const avgFieldLength = this._avgFieldLength[fieldId];
      for (const docId of fieldTermFreqs.keys()) {
        if (!this._documentIds.has(docId)) {
          this.removeTerm(fieldId, docId, derivedTerm);
          matchingFields -= 1;
          continue;
        }
        const docBoost = boostDocumentFn ? boostDocumentFn(this._documentIds.get(docId), derivedTerm, this._storedFields.get(docId)) : 1;
        if (!docBoost)
          continue;
        const termFreq = fieldTermFreqs.get(docId);
        const fieldLength = this._fieldLength.get(docId)[fieldId];
        const rawScore = calcBM25Score(termFreq, matchingFields, this._documentCount, fieldLength, avgFieldLength, bm25params);
        const weightedScore = termWeight * termBoost * fieldBoost * docBoost * rawScore;
        const result2 = results.get(docId);
        if (result2) {
          result2.score += weightedScore;
          assignUniqueTerm(result2.terms, sourceTerm);
          const match2 = getOwnProperty(result2.match, derivedTerm);
          if (match2) {
            match2.push(field);
          } else {
            result2.match[derivedTerm] = [field];
          }
        } else {
          results.set(docId, {
            score: weightedScore,
            terms: [sourceTerm],
            match: { [derivedTerm]: [field] }
          });
        }
      }
    }
    return results;
  }
  /**
   * @ignore
   */
  addTerm(fieldId, documentId, term) {
    const indexData = this._index.fetch(term, createMap);
    let fieldIndex = indexData.get(fieldId);
    if (fieldIndex == null) {
      fieldIndex = /* @__PURE__ */ new Map();
      fieldIndex.set(documentId, 1);
      indexData.set(fieldId, fieldIndex);
    } else {
      const docs = fieldIndex.get(documentId);
      fieldIndex.set(documentId, (docs || 0) + 1);
    }
  }
  /**
   * @ignore
   */
  removeTerm(fieldId, documentId, term) {
    if (!this._index.has(term)) {
      this.warnDocumentChanged(documentId, fieldId, term);
      return;
    }
    const indexData = this._index.fetch(term, createMap);
    const fieldIndex = indexData.get(fieldId);
    if (fieldIndex == null || fieldIndex.get(documentId) == null) {
      this.warnDocumentChanged(documentId, fieldId, term);
    } else if (fieldIndex.get(documentId) <= 1) {
      if (fieldIndex.size <= 1) {
        indexData.delete(fieldId);
      } else {
        fieldIndex.delete(documentId);
      }
    } else {
      fieldIndex.set(documentId, fieldIndex.get(documentId) - 1);
    }
    if (this._index.get(term).size === 0) {
      this._index.delete(term);
    }
  }
  /**
   * @ignore
   */
  warnDocumentChanged(shortDocumentId, fieldId, term) {
    for (const fieldName of Object.keys(this._fieldIds)) {
      if (this._fieldIds[fieldName] === fieldId) {
        this._options.logger("warn", `MiniSearch: document with ID ${this._documentIds.get(shortDocumentId)} has changed before removal: term "${term}" was not present in field "${fieldName}". Removing a document after it has changed can corrupt the index!`, "version_conflict");
        return;
      }
    }
  }
  /**
   * @ignore
   */
  addDocumentId(documentId) {
    const shortDocumentId = this._nextId;
    this._idToShortId.set(documentId, shortDocumentId);
    this._documentIds.set(shortDocumentId, documentId);
    this._documentCount += 1;
    this._nextId += 1;
    return shortDocumentId;
  }
  /**
   * @ignore
   */
  addFields(fields) {
    for (let i = 0; i < fields.length; i++) {
      this._fieldIds[fields[i]] = i;
    }
  }
  /**
   * @ignore
   */
  addFieldLength(documentId, fieldId, count, length) {
    let fieldLengths = this._fieldLength.get(documentId);
    if (fieldLengths == null)
      this._fieldLength.set(documentId, fieldLengths = []);
    fieldLengths[fieldId] = length;
    const averageFieldLength = this._avgFieldLength[fieldId] || 0;
    const totalFieldLength = averageFieldLength * count + length;
    this._avgFieldLength[fieldId] = totalFieldLength / (count + 1);
  }
  /**
   * @ignore
   */
  removeFieldLength(documentId, fieldId, count, length) {
    if (count === 1) {
      this._avgFieldLength[fieldId] = 0;
      return;
    }
    const totalFieldLength = this._avgFieldLength[fieldId] * count - length;
    this._avgFieldLength[fieldId] = totalFieldLength / (count - 1);
  }
  /**
   * @ignore
   */
  saveStoredFields(documentId, doc) {
    const { storeFields, extractField } = this._options;
    if (storeFields == null || storeFields.length === 0) {
      return;
    }
    let documentFields = this._storedFields.get(documentId);
    if (documentFields == null)
      this._storedFields.set(documentId, documentFields = {});
    for (const fieldName of storeFields) {
      const fieldValue = extractField(doc, fieldName);
      if (fieldValue !== void 0)
        documentFields[fieldName] = fieldValue;
    }
  }
};
MiniSearch.wildcard = /* @__PURE__ */ Symbol("*");
var getOwnProperty = (object2, property) => Object.prototype.hasOwnProperty.call(object2, property) ? object2[property] : void 0;
var combinators = {
  [OR]: (a, b) => {
    for (const docId of b.keys()) {
      const existing = a.get(docId);
      if (existing == null) {
        a.set(docId, b.get(docId));
      } else {
        const { score, terms, match: match2 } = b.get(docId);
        existing.score = existing.score + score;
        existing.match = Object.assign(existing.match, match2);
        assignUniqueTerms(existing.terms, terms);
      }
    }
    return a;
  },
  [AND]: (a, b) => {
    const combined = /* @__PURE__ */ new Map();
    for (const docId of b.keys()) {
      const existing = a.get(docId);
      if (existing == null)
        continue;
      const { score, terms, match: match2 } = b.get(docId);
      assignUniqueTerms(existing.terms, terms);
      combined.set(docId, {
        score: existing.score + score,
        terms: existing.terms,
        match: Object.assign(existing.match, match2)
      });
    }
    return combined;
  },
  [AND_NOT]: (a, b) => {
    for (const docId of b.keys())
      a.delete(docId);
    return a;
  }
};
var defaultBM25params = { k: 1.2, b: 0.7, d: 0.5 };
var calcBM25Score = (termFreq, matchingCount, totalCount, fieldLength, avgFieldLength, bm25params) => {
  const { k, b, d } = bm25params;
  const invDocFreq = Math.log(1 + (totalCount - matchingCount + 0.5) / (matchingCount + 0.5));
  return invDocFreq * (d + termFreq * (k + 1) / (termFreq + k * (1 - b + b * fieldLength / avgFieldLength)));
};
var termToQuerySpec = (options) => (term, i, terms) => {
  const fuzzy = typeof options.fuzzy === "function" ? options.fuzzy(term, i, terms) : options.fuzzy || false;
  const prefix = typeof options.prefix === "function" ? options.prefix(term, i, terms) : options.prefix === true;
  const termBoost = typeof options.boostTerm === "function" ? options.boostTerm(term, i, terms) : 1;
  return { term, fuzzy, prefix, termBoost };
};
var defaultOptions = {
  idField: "id",
  extractField: (document2, fieldName) => document2[fieldName],
  stringifyField: (fieldValue, fieldName) => fieldValue.toString(),
  tokenize: (text2) => text2.split(SPACE_OR_PUNCTUATION),
  processTerm: (term) => term.toLowerCase(),
  fields: void 0,
  searchOptions: void 0,
  storeFields: [],
  logger: (level, message) => {
    if (typeof (console === null || console === void 0 ? void 0 : console[level]) === "function")
      console[level](message);
  },
  autoVacuum: true
};
var defaultSearchOptions = {
  combineWith: OR,
  prefix: false,
  fuzzy: false,
  maxFuzzy: 6,
  boost: {},
  weights: { fuzzy: 0.45, prefix: 0.375 },
  bm25: defaultBM25params
};
var defaultAutoSuggestOptions = {
  combineWith: AND,
  prefix: (term, i, terms) => i === terms.length - 1
};
var defaultVacuumOptions = { batchSize: 1e3, batchWait: 10 };
var defaultVacuumConditions = { minDirtFactor: 0.1, minDirtCount: 20 };
var defaultAutoVacuumOptions = { ...defaultVacuumOptions, ...defaultVacuumConditions };
var assignUniqueTerm = (target, term) => {
  if (!target.includes(term))
    target.push(term);
};
var assignUniqueTerms = (target, source) => {
  for (const term of source) {
    if (!target.includes(term))
      target.push(term);
  }
};
var byScore = ({ score: a }, { score: b }) => b - a;
var createMap = () => /* @__PURE__ */ new Map();
var objectToNumericMap = (object2) => {
  const map = /* @__PURE__ */ new Map();
  for (const key of Object.keys(object2)) {
    map.set(parseInt(key, 10), object2[key]);
  }
  return map;
};
var objectToNumericMapAsync = async (object2) => {
  const map = /* @__PURE__ */ new Map();
  let count = 0;
  for (const key of Object.keys(object2)) {
    map.set(parseInt(key, 10), object2[key]);
    if (++count % 1e3 === 0) {
      await wait(0);
    }
  }
  return map;
};
var wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
var SPACE_OR_PUNCTUATION = /[\n\r\p{Z}\p{P}]+/u;

// src/policies/address.ts
var TAGS = "tags/";
var STEPS = "steps";
var SEARCH = "search";
var KEPT = /[A-Za-z0-9\-._~/]/;
function readTarget(hash) {
  const rest = hash.startsWith("#") ? hash.slice(1) : hash;
  if (rest.startsWith("#")) {
    return { kind: "element", id: decoded2(rest.slice(1)) };
  }
  const split = rest.indexOf("#");
  const name2 = split < 0 ? rest : rest.slice(0, split);
  const anchor = split < 0 || split === rest.length - 1 ? null : decoded2(rest.slice(split + 1));
  return { kind: "page", page: pageNamed(name2), anchor };
}
function targetAddress(page, anchor = null) {
  const named3 = `#${encodedName(page)}`;
  return anchor === null ? named3 : `${named3}#${encoded(anchor)}`;
}
function pageNamed(raw) {
  if (raw === SEARCH || raw.startsWith(`${SEARCH}?`)) {
    return { kind: "search", words: new URLSearchParams(raw.slice(SEARCH.length + 1)).get("q") ?? "" };
  }
  return pageByName(decoded2(raw));
}
function pageByName(name2) {
  if (name2 === "") {
    return { kind: "home" };
  }
  if (name2 === TAGS) {
    return { kind: "tags" };
  }
  if (name2.startsWith(TAGS)) {
    return { kind: "tag", tag: name2.slice(TAGS.length) };
  }
  if (name2 === STEPS) {
    return { kind: "steps" };
  }
  return name2.endsWith("/") ? { kind: "folder", folder: name2 } : { kind: "feature", page: name2 };
}
function encodedName(page) {
  switch (page.kind) {
    case "home":
      return "";
    case "folder":
      return encoded(page.folder);
    case "feature":
      return encoded(page.page);
    case "tags":
      return TAGS;
    case "tag":
      return encoded(`${TAGS}${page.tag}`);
    case "steps":
      return STEPS;
    case "search":
      return page.words === "" ? SEARCH : `${SEARCH}?${new URLSearchParams({ q: page.words }).toString()}`;
  }
}
function encoded(text2) {
  return [...text2].map(encodedCharacter).join("");
}
function encodedCharacter(character) {
  if (KEPT.test(character)) {
    return character;
  }
  return [...new TextEncoder().encode(character)].map(percentByte).join("");
}
function percentByte(byte) {
  return `%${byte.toString(16).toUpperCase().padStart(2, "0")}`;
}
function decoded2(text2) {
  try {
    return decodeURIComponent(text2);
  } catch {
    return text2;
  }
}

// src/policies/search_index.ts
var INDEX_OPTIONS = {
  fields: ["name", "tags", "steps", "cells"],
  extractField: (document2, field) => {
    const value = document2[field];
    return Array.isArray(value) ? value.join(" ") : value;
  }
};
var SEARCH_OPTIONS = {
  combineWith: "AND",
  prefix: true,
  fuzzy: 0.2,
  boost: { name: 3, tags: 2, steps: 1, cells: 1 }
};
var BETWEEN_WORDS = /([\n\r\p{Z}\p{P}]+)/u;
function indexOf(documents) {
  const search2 = new MiniSearch(INDEX_OPTIONS);
  search2.addAll(documents.map((document2, id) => ({ ...document2, id })));
  return {
    search: (text2) => search2.search(text2, SEARCH_OPTIONS).flatMap((match2) => found(documents, match2))
  };
}
function found(documents, match2) {
  const document2 = documents[match2.id];
  if (document2 === void 0) {
    return [];
  }
  const matched = /* @__PURE__ */ new Map();
  for (const [term, fields] of Object.entries(match2.match)) {
    for (const field of fields) {
      matched.set(field, (matched.get(field) ?? /* @__PURE__ */ new Set()).add(term));
    }
  }
  return [{ document: document2, matched }];
}
function marked(text2, matched) {
  const runs = [];
  for (const part2 of text2.split(BETWEEN_WORDS)) {
    if (part2 === "") {
      continue;
    }
    const isMatch = matched?.has(part2.toLowerCase()) ?? false;
    const last2 = runs.at(-1);
    if (last2 !== void 0 && !last2.marked && !isMatch) {
      runs[runs.length - 1] = { text: last2.text + part2, marked: false };
    } else {
      runs.push({ text: part2, marked: isMatch });
    }
  }
  return runs;
}
function holdsMatch(runs) {
  return runs.some((run2) => run2.marked);
}
var WRITTEN_ADDRESSES = {
  entry: (document2) => {
    const page = document2.page.split("/").map(encodeURIComponent).join("/");
    return document2.anchor === "" ? page : `${page}#${encodeURIComponent(document2.anchor)}`;
  },
  results: (words) => `search.html?${new URLSearchParams({ q: words }).toString()}`
};
var SHELL_ADDRESSES = {
  entry: (document2) => targetAddress(pageByName(document2.page), document2.anchor === "" ? null : document2.anchor),
  results: (words) => targetAddress({ kind: "search", words })
};

// src/policies/block_state.ts
var EXPANDED2 = "expanded";
var COLLAPSED2 = "collapsed";
function memoryKey(pagePath, anchor) {
  return `gherkin-glass:${pagePath}#${anchor}`;
}
function inlineMemoryKey(pagePath, anchor) {
  return `gherkin-glass:inline:${pagePath}#${anchor}`;
}
function remembered(expanded) {
  return expanded ? EXPANDED2 : COLLAPSED2;
}
function startState(configured, kept) {
  if (kept === EXPANDED2) {
    return true;
  }
  if (kept === COLLAPSED2) {
    return false;
  }
  return configured;
}
function formStart(written, expandedWhenUnchecked2, inScenario, inline) {
  if (expandedWhenUnchecked2 === null) {
    return written;
  }
  return inline ? inScenario : expandedWhenUnchecked2;
}
function switchedState(inScenario, inline) {
  return inline === inScenario;
}

// src/services/collapsing.ts
function startCollapsing(page, memory, pagePath, option) {
  const blocks = page.blocks();
  const followingTheForm = blocks.filter((block2) => block2.expandedWhenUnchecked !== null);
  let inline = false;
  const key = (block2) => inline && block2.expandedWhenUnchecked !== null ? inlineMemoryKey(pagePath, block2.anchor) : memoryKey(pagePath, block2.anchor);
  const set = (block2, expanded) => {
    block2.setExpanded(expanded);
    memory.set(key(block2), remembered(expanded));
  };
  const settle = (block2) => {
    const start = formStart(block2.isExpanded(), block2.expandedWhenUnchecked, block2.inScenario, inline);
    block2.setExpanded(startState(start, memory.get(key(block2))));
  };
  const showForm = (unchosen) => {
    if (option !== null) {
      inline = startForm(unchosen, memory.get(formKey(option.siteRoot)));
      option.backgrounds.setInline(inline);
    }
  };
  const reveal = (anchor) => {
    let block2 = page.holder(anchor);
    while (block2 !== null) {
      if (!block2.isExpanded()) {
        set(block2, true);
      }
      block2 = block2.parent;
    }
  };
  showForm(option?.backgrounds.writtenInline() ?? false);
  blocks.forEach(settle);
  const target = page.target();
  if (target !== null) {
    reveal(target);
  }
  page.onToggle((block2) => set(block2, !block2.isExpanded()));
  page.onCollapseAll(() => blocks.forEach((block2) => set(block2, false)));
  page.onExpandAll(() => blocks.forEach((block2) => set(block2, true)));
  page.onFollow(reveal);
  if (option !== null) {
    option.backgrounds.onSwitch((switched) => {
      inline = switched;
      option.backgrounds.setInline(inline);
      memory.set(formKey(option.siteRoot), rememberedForm(inline));
      followingTheForm.forEach((block2) => set(block2, switchedState(block2.inScenario, inline)));
    });
    option.backgrounds.onReturn(() => {
      const shown = inline;
      showForm(shown);
      if (inline !== shown) {
        followingTheForm.forEach(settle);
      }
    });
  }
  return reveal;
}

// src/services/folder_tree.ts
async function startFolderTree(source, toggle2, view, width, memory, place) {
  const controls = startTreeControls(toggle2, width, memory, place.sitePath);
  const items = await source.load();
  if (items === null) {
    controls.disable();
    return;
  }
  buildTree(view, memory, place.sitePath, items, place.page === null ? [] : pathTo(items, place.page));
}
function buildTree(view, memory, sitePath, items, path) {
  view.build(items, path[0] ?? null, (address) => startExpanded(path.includes(address), memory.get(itemKey(sitePath, address))));
  view.onToggle((address, expanded) => memory.set(itemKey(sitePath, address), rememberedExpanded(expanded)));
}
function startTreeControls(toggle2, width, memory, sitePath) {
  let checked = startChecked(toggle2.configured(), memory.get(shownKey(sitePath)));
  let open = false;
  let narrow = width.narrow();
  let usable = true;
  const show = () => {
    toggle2.setChecked(usable && checked);
    toggle2.setOpen(usable && open);
    toggle2.setShown(usable && treeShown(narrow, checked, open));
  };
  show();
  toggle2.onChange((chosen) => {
    checked = chosen;
    memory.set(shownKey(sitePath), rememberedShown(chosen));
    show();
  });
  toggle2.onPress(() => {
    open = !open;
    show();
  });
  toggle2.onEscape(() => {
    if (narrow && open) {
      open = false;
      show();
      toggle2.focusButton();
    }
  });
  width.onChange((now) => {
    const focus = toggle2.focus();
    narrow = now;
    open = false;
    show();
    if (focusHidden(focus, narrow, treeShown(narrow, checked, open))) {
      if (narrow) {
        toggle2.focusButton();
      } else {
        toggle2.focusCheckbox();
      }
    }
  });
  return {
    disable: () => {
      usable = false;
      toggle2.disable();
      show();
    }
  };
}

// src/policies/search_documents.ts
function searchDocuments(features) {
  return features.flatMap(({ entry, object: object2 }) => {
    const feature2 = object2.feature;
    const document2 = (block2, rule, outline, tags) => ({
      kind: block2.kind,
      name: block2.name,
      feature: block2 === feature2 ? null : feature2.name,
      rule: rule === null ? null : rule.name,
      outline: outline === null ? null : outline.name,
      place: outline === null ? null : block2.place,
      steps: block2.steps.map((step2) => step2.text),
      tags,
      cells: block2.table?.flat() ?? [],
      page: entry.page,
      anchor: block2 === feature2 ? "" : block2.anchor
    });
    const inside = (blocks, rule) => blocks.flatMap(
      (block2) => block2.kind === "rule" ? [document2(block2, null, null, block2.tags), ...inside(block2.blocks, block2)] : [document2(block2, rule, null, outlineTags(block2)), ...block2.examples.map((examples) => document2(examples, rule, block2, examples.tags))]
    );
    return [document2(feature2, null, null, feature2.tags), ...inside(feature2.blocks, null)];
  });
}
function outlineTags(block2) {
  return [.../* @__PURE__ */ new Set([...block2.tags, ...block2.examples.flatMap((examples) => examples.tags)])];
}

// src/services/loaded_features.ts
function loadedFeatures(store, descriptor) {
  return Promise.all(descriptor.features.map(async (entry) => ({ entry, object: await store.object(entry.object) })));
}
function storeDocuments(store) {
  return { load: async () => searchDocuments(await loadedFeatures(store, await store.descriptor())) };
}

// src/policies/heading_words.ts
function isolated(text2) {
  return [{ text: text2, isolated: true }];
}
function plain(text2) {
  return text2 === "" ? [] : [{ text: text2, isolated: false }];
}
function pieces(words, id, args, slots) {
  const marks = slots.map((_, place) => String.fromCodePoint(57600 + place));
  const sentence = words.text(id, { ...args, ...Object.fromEntries(slots.map((slot, place) => [slot, marks[place] ?? ""])) });
  return [...sentence.split(/([\u{e100}-\u{e1ff}])/u)].flatMap((piece) => {
    const place = marks.indexOf(piece);
    const slot = slots[place];
    return slot !== void 0 ? [{ slot }] : piece === "" ? [] : [{ words: piece }];
  });
}
function wordsAround(words, id, args, texts, slot) {
  const mark = "";
  const parts = words.parts(id, args, { ...texts, [slot]: plain(mark) });
  const at = parts.findIndex((part2) => part2.text === mark);
  return at < 0 ? { before: parts, after: [] } : { before: parts.slice(0, at), after: parts.slice(at + 1) };
}
function textOf2(parts) {
  return parts.map((part2) => part2.text).join("");
}
function heading(words, kind, name2) {
  return named(words, kind, name2, words.text("unnamed-heading", { kind }));
}
function examplesHeading(words, name2, place) {
  return named(words, "examples", name2, words.text("numbered-examples", { place }));
}
function blockHeading(words, block2) {
  return block2.kind === "examples" ? examplesHeading(words, block2.name, block2.place ?? 1) : heading(words, block2.kind, block2.name);
}
function named(words, kind, name2, unnamed) {
  return name2 === "" ? plain(unnamed) : words.parts("named-heading", { kind }, { name: isolated(name2) });
}
function keywordNote(words, block2) {
  if (!block2.note) {
    return null;
  }
  const mark = "";
  const sentence = words.text("keyword-note", { kind: block2.kind, opener: mark });
  const split = sentence.indexOf(mark);
  return {
    before: split < 0 ? sentence : sentence.slice(0, split),
    opener: words.parts("keyword-note-opener", {}, { keyword: isolated(block2.keyword) }),
    after: split < 0 ? "" : sentence.slice(split + mark.length)
  };
}

// src/policies/block_views.ts
var EMPTY = {
  expandedWhenUnchecked: null,
  anchor: null,
  note: null,
  tags: [],
  description: [],
  backgrounds: [],
  stepsHeading: null,
  steps: [],
  examples: null,
  blocks: []
};
function tagViews(context, tags) {
  return tags.map((tag2) => {
    const entry = context.tags.get(tag2);
    return { text: tag2, address: targetAddress(pageByName(entry?.page ?? "tags/")), link: entry?.link ?? null };
  });
}
function descriptionLines(description) {
  return description === "" ? [] : description.split("\n");
}
function tableView(table2) {
  const [header, ...rows] = table2 ?? [];
  return header === void 0 ? null : { header, rows };
}
function stepViews(block2) {
  return block2.steps.map((step2) => ({ keyword: step2.keyword, parts: step2.parts, table: tableView(step2.table), docString: step2.doc_string }));
}
function backgroundView(context, background, level, startsScenarios) {
  const follows = startsScenarios && background.steps.length > 0;
  const collapsed = follows && context.start.inline_backgrounds || context.start.collapse_scenarios;
  return {
    ...EMPTY,
    className: "ghg-background",
    level,
    id: background.anchor,
    anchor: background.anchor,
    heading: blockHeading(context.words, background),
    expanded: !collapsed,
    expandedWhenUnchecked: follows ? !context.start.collapse_scenarios : null,
    note: keywordNote(context.words, background),
    description: descriptionLines(background.description),
    steps: stepViews(background)
  };
}
function scenarioView(context, scenario, level, backgrounds, examples) {
  const inScenario = backgrounds.filter((background) => background.block.steps.length > 0).map((background, place) => backgroundInScenario(context, background, `${scenario.anchor}_background_${place + 1}`, level + 1));
  const steps = stepViews(scenario);
  return {
    ...EMPTY,
    className: "ghg-scenario",
    level,
    id: scenario.anchor,
    anchor: scenario.anchor,
    heading: blockHeading(context.words, scenario),
    expanded: !context.start.collapse_scenarios,
    note: keywordNote(context.words, scenario),
    tags: tagViews(context, scenario.tags),
    description: descriptionLines(scenario.description),
    backgrounds: inScenario,
    stepsHeading: inScenario.length > 0 && steps.length > 0 ? { level: level + 1, text: context.words.text("from-scenario") } : null,
    steps,
    examples: examples(scenario, level + 1)
  };
}
function backgroundInScenario(context, background, id, level) {
  return {
    ...EMPTY,
    className: "ghg-background-in-scenario",
    level,
    id,
    heading: plain(background.heading),
    expanded: context.start.inline_backgrounds,
    expandedWhenUnchecked: false,
    steps: stepViews(background.block)
  };
}
function ruleView(context, rule, featureBackground, examples) {
  const scenarios = rule.blocks.filter(isScenario);
  const background = rule.blocks.find((block2) => block2.kind === "background") ?? null;
  const starting = [featureBackground, background && { block: background, heading: context.words.text("from-rule-background") }].filter((found2) => found2 !== null);
  return {
    ...EMPTY,
    className: "ghg-rule",
    level: 2,
    id: rule.anchor,
    anchor: rule.anchor,
    heading: blockHeading(context.words, rule),
    expanded: !context.start.collapse_rules,
    note: keywordNote(context.words, rule),
    tags: tagViews(context, rule.tags),
    description: descriptionLines(rule.description),
    blocks: [
      ...background === null ? [] : [backgroundView(context, background, 3, scenarios.length > 0)],
      ...scenarios.map((scenario) => scenarioView(context, scenario, 3, starting, examples))
    ]
  };
}
function outsideRulesView(context, scenarios) {
  return { ...EMPTY, className: "ghg-outside-rules", level: 2, id: "outside_rules", heading: plain(context.words.text("outside-rules")), expanded: !context.start.collapse_rules, blocks: scenarios };
}
function isScenario(block2) {
  return block2.kind === "scenario" || block2.kind === "scenario-outline";
}
function outlineName(words, outline) {
  return outline.name === "" ? heading(words, "scenario-outline", "") : [{ text: outline.name, isolated: true }];
}

// src/policies/examples_views.ts
function examplesView(context, outline, level) {
  if (outline.examples.length === 0) {
    return null;
  }
  const hasRows = outline.examples.some((examples) => examples.runs.length > 0);
  return {
    runsParagraph: hasRows ? { id: `${outline.anchor}_scenarios`, text: context.words.text("runs-for-each-row") } : null,
    items: outline.examples.map((examples) => examplesItem2(context, outline, examples, level))
  };
}
function examplesItem2(context, outline, examples, level) {
  const heading2 = blockHeading(context.words, examples);
  const note = keywordNote(context.words, examples);
  const table2 = tableView(examples.table);
  const description = descriptionLines(examples.description);
  return {
    id: examples.anchor,
    label: examples.runs.length > 0 ? heading2 : context.words.parts("nothing-to-run", {}, { heading: heading2 }),
    holdsNothing: note === null && table2 === null && examples.tags.length === 0 && description.length === 0,
    note,
    tags: tagViews(context, examples.tags),
    description,
    table: table2,
    runs: examples.runs.length === 0 ? null : runsView(context, outline, examples, level)
  };
}
function runsView(context, outline, examples, level) {
  const [columns = [], ...rows] = examples.table ?? [];
  return {
    ...EMPTY,
    className: "ghg-examples-as-scenarios",
    level,
    id: `${examples.anchor}_runs`,
    heading: plain(context.words.text("runs-one-per-row")),
    expanded: false,
    blocks: examples.runs.map((run2, row) => runView(context, outline, run2, columns, rows[row] ?? [], level + 1))
  };
}
function runView(context, outline, run2, columns, cells, level) {
  return {
    ...EMPTY,
    className: "ghg-scenario",
    level,
    id: run2.anchor,
    heading: context.words.parts("example-scenario-name", {}, { name: outlineName(context.words, outline), values: rowValues(context, columns, cells) }),
    expanded: !context.start.collapse_scenarios,
    steps: stepViews(run2)
  };
}
function rowValues(context, columns, cells) {
  const separator = plain(context.words.text("example-values-separator"));
  return columns.flatMap((column, place) => [
    ...place === 0 ? [] : separator,
    ...context.words.parts("example-value", {}, { column: isolated(column), value: isolated(cells[place] ?? "") })
  ]);
}

// src/policies/feature_view.ts
function featureView(words, descriptor, entry, object2) {
  const feature2 = object2.feature;
  const rules = feature2.blocks.filter((block2) => block2.kind === "rule");
  const scenarios = feature2.blocks.filter(isScenario);
  const background = feature2.blocks.find((block2) => block2.kind === "background") ?? null;
  const hasBackground = background !== null || rules.some((rule) => rule.blocks.some((block2) => block2.kind === "background"));
  const context = {
    words,
    start: descriptor.start,
    tags: new Map(descriptor.tags.map((tag2) => [tag2.tag, tag2]))
  };
  const featureBackground = background && { block: background, heading: words.text("from-background") };
  const startsScenarios = scenarios.length > 0 || rules.some((rule) => rule.blocks.some(isScenario));
  const scenarioLevel = rules.length > 0 && scenarios.length > 0 ? 3 : 2;
  const scenarioViews = scenarios.map((scenario) => scenarioView(context, scenario, scenarioLevel, featureBackground === null ? [] : [featureBackground], (outline, level) => examplesView(context, outline, level)));
  const blocks = [
    ...background === null ? [] : [backgroundView(context, background, 2, startsScenarios)],
    ...scenarioLevel === 3 ? [outsideRulesView(context, scenarioViews)] : scenarioViews,
    ...rules.map((rule) => ruleView(context, rule, featureBackground, (outline, level) => examplesView(context, outline, level)))
  ];
  const featureHeading2 = heading(words, "feature", feature2.name);
  return {
    title: words.text("page-title", { heading: textOf2(featureHeading2), path: entry.path }),
    lang: object2.lang,
    heading: featureHeading2,
    note: keywordNote(words, feature2),
    tags: tagViews(context, feature2.tags),
    description: descriptionLines(feature2.description),
    source: entry.source === null ? null : { address: entry.source, words: words.parts("source-link", {}, { path: isolated(entry.path) }) },
    contents: contentsOf(words, entry.page, feature2),
    controls: blocks.length === 0 ? null : {
      collapseAll: words.text("collapse-all"),
      expandAll: words.text("expand-all"),
      backgrounds: hasBackground ? { label: words.text("inline-backgrounds"), checked: descriptor.start.inline_backgrounds, enabled: startsWithBackground(feature2) } : null
    },
    blocks,
    dialog: hasNote(feature2) ? { title: words.text("note-dialog-title"), text: words.text("note-dialog-text"), close: words.text("note-dialog-close") } : null
  };
}
function contentsOf(words, page, feature2) {
  const link2 = (anchor, parts, entries = []) => ({ address: targetAddress({ kind: "feature", page }, anchor), heading: parts, entries });
  const scenarioEntry = (scenario) => link2(scenario.anchor, heading(words, scenario.kind, scenario.name));
  const rules = feature2.blocks.filter((block2) => block2.kind === "rule");
  const scenarios = feature2.blocks.filter(isScenario);
  const ruleEntries = rules.map((rule) => link2(rule.anchor, heading(words, "rule", rule.name), rule.blocks.filter(isScenario).map(scenarioEntry)));
  if (rules.length > 0 && scenarios.length > 0) {
    return [link2("outside_rules", [{ text: words.text("outside-rules"), isolated: false }], scenarios.map(scenarioEntry)), ...ruleEntries];
  }
  return [...scenarios.map(scenarioEntry), ...ruleEntries];
}
function startsWithBackground(feature2) {
  const withSteps = (block2) => block2.kind === "background" && block2.steps.length > 0;
  const featureSteps = feature2.blocks.some(withSteps);
  return featureSteps && feature2.blocks.some(isScenario) || feature2.blocks.some((rule) => rule.kind === "rule" && rule.blocks.some(isScenario) && (featureSteps || rule.blocks.some(withSteps)));
}
function hasNote(block2) {
  return block2.note || block2.blocks.some(hasNote) || block2.examples.some(hasNote);
}

// src/policies/site_folders.ts
function siteFolders(features) {
  const folders = /* @__PURE__ */ new Map([["", { name: "", folders: [], features: [] }]]);
  const folderNamed = (name2) => {
    const known = folders.get(name2);
    if (known !== void 0) {
      return known;
    }
    const made = { name: name2, folders: [], features: [] };
    folders.set(name2, made);
    folderNamed(parentOf(name2)).folders.push(name2);
    return made;
  };
  for (const feature2 of features) {
    folderNamed(parentOf(feature2.entry.page)).features.push(feature2);
  }
  return folders;
}
function parentOf(name2) {
  const inner = name2.endsWith("/") ? name2.slice(0, -1) : name2;
  const slash = inner.lastIndexOf("/");
  return slash === -1 ? "" : inner.slice(0, slash + 1);
}
function folderName(name2) {
  return name2.slice(parentOf(name2).length, -1);
}
function folderPath(name2) {
  return name2.slice(0, -1);
}
function scenarioCount(feature2) {
  return feature2.blocks.reduce((count, block2) => count + (isScenario(block2) ? 1 : block2.kind === "rule" ? block2.blocks.filter(isScenario).length : 0), 0);
}
function ruleCount(feature2) {
  return feature2.blocks.filter((block2) => block2.kind === "rule").length;
}

// src/policies/folder_view.ts
function folderView(words, descriptor, features, name2) {
  const folders = siteFolders(features);
  const folder2 = folders.get(name2);
  if (folder2 === void 0) {
    return null;
  }
  const home2 = name2 === "";
  return {
    title: home2 ? descriptor.title : words.text("folder-page-title", { folder: folderPath(name2), site: descriptor.title }),
    heading: home2 ? descriptor.title : folderName(name2),
    totals: home2 ? totals(words, features, folders.size - 1) : [],
    tagIndex: home2 ? { address: targetAddress({ kind: "tags" }), text: words.text("tag-index-title") } : null,
    sentences: home2 ? { address: targetAddress({ kind: "steps" }), text: words.text("sentences-title") } : null,
    nothingGiven: home2 && descriptor.only && descriptor.features.length === 0 ? words.text("only-nothing-given") : null,
    folders: folder2.folders.length === 0 ? null : { heading: words.text("folders"), links: folder2.folders.map((inner) => ({ address: targetAddress({ kind: "folder", folder: inner }), text: folderName(inner) })) },
    features: folder2.features.length === 0 ? null : { heading: words.text("features"), entries: featureEntries(words, folder2) }
  };
}
function totals(words, features, folders) {
  const sum = (count) => features.reduce((total, feature2) => total + count(feature2.object.feature), 0);
  return [
    words.text("folder-count", { count: folders }),
    words.text("feature-count", { count: features.length }),
    words.text("rule-count", { count: sum(ruleCount) }),
    words.text("scenario-count", { count: sum(scenarioCount) })
  ];
}
function featureEntries(words, folder2) {
  return folder2.features.map(({ entry, object: object2 }) => {
    const featureHeading2 = heading(words, "feature", object2.feature.name);
    const count = scenarioCount(object2.feature);
    const contents = contentsOf(words, entry.page, object2.feature);
    return {
      address: targetAddress({ kind: "feature", page: entry.page }),
      heading: featureHeading2,
      lang: object2.lang,
      count: words.text("scenario-count", { count }),
      countName: contents.length === 0 ? null : words.text("feature-scenarios", { count, feature: textOf2(featureHeading2) }),
      contents
    };
  });
}

// src/policies/other_versions.ts
function otherObjects(index, page, shown) {
  return (index.pages.find((entry) => entry.page === page)?.objects ?? []).map((entry) => entry.object).filter((object2) => object2 !== shown);
}
function pageInVersions(words, descriptor, list2, index, page, shown, others) {
  const objects = index.pages.find((entry) => entry.page === page)?.objects ?? [];
  const versions = [...list2.development, ...list2.releases, ...list2.pre_releases].filter((version) => version !== descriptor.version);
  const shownHashes = hashesOf(shown.object.feature);
  const compared = versions.map((version) => {
    const name2 = objects.find((entry) => entry.versions.includes(version))?.object;
    const object2 = name2 === shown.name ? shown.object : name2 === void 0 ? void 0 : others.get(name2);
    if (object2 === void 0) {
      return { version, comparison: { kind: "missing" } };
    }
    const hashes = hashesOf(object2.feature);
    return { version, comparison: { kind: "page", hashes, same: sameHashes(shownHashes, hashes) } };
  });
  const link2 = (version, target, anchor) => ({
    address: `${descriptor.preview === null ? "../" : "../../"}${version}/index.html${target === null ? "" : targetAddress(target, anchor)}`,
    text: newer(list2, version, descriptor.version) ? words.text("other-version-newer", { version }) : version
  });
  const feature2 = { kind: "feature", page };
  const pageGroups = groupsOf(words, {
    changed: compared.flatMap(({ version, comparison }) => comparison.kind === "page" && !comparison.same ? [{ ...link2(version, feature2, null), item: changes(words, shownHashes, comparison.hashes) }] : []),
    missing: compared.flatMap(({ version, comparison }) => comparison.kind === "missing" ? [{ ...link2(version, null, null), item: LINK_ALONE }] : []),
    unchanged: compared.flatMap(({ version, comparison }) => comparison.kind === "page" && comparison.same ? [{ ...link2(version, feature2, null), item: LINK_ALONE }] : [])
  });
  const blocks = new Map(
    [...shownHashes].flatMap(([anchor, hash]) => {
      if (anchor === "") {
        return [];
      }
      const inVersion = (comparison) => {
        const other = comparison.kind === "page" ? comparison.hashes.get(anchor) : void 0;
        return other === void 0 ? "missing" : other === hash ? "unchanged" : "changed";
      };
      const blockLink = (version, comparison) => {
        const state = inVersion(comparison);
        return { ...link2(version, comparison.kind === "missing" ? null : feature2, state === "missing" ? null : anchor), item: LINK_ALONE };
      };
      const view = groupsOf(words, {
        changed: compared.filter(({ comparison }) => inVersion(comparison) === "changed").map(({ version, comparison }) => blockLink(version, comparison)),
        missing: compared.filter(({ comparison }) => inVersion(comparison) === "missing").map(({ version, comparison }) => blockLink(version, comparison)),
        unchanged: compared.filter(({ comparison }) => inVersion(comparison) === "unchanged").map(({ version, comparison }) => blockLink(version, comparison))
      });
      return view === null ? [] : [[anchor, { ...view, label: words.text("other-versions-block") }]];
    })
  );
  return { page: pageGroups && { ...pageGroups, label: words.text("other-versions-page") }, blocks };
}
function groupsOf(words, links) {
  const groups = [
    { label: words.text("other-versions-changed"), links: links.changed },
    { label: words.text("other-versions-missing"), links: links.missing },
    { label: words.text("other-versions-unchanged"), links: links.unchanged }
  ].filter((group) => group.links.length > 0);
  return groups.length === 0 ? null : { groups };
}
function hashesOf(feature2) {
  const counted = (block2) => block2.kind === "rule" ? [[block2.anchor, block2.hash], ...block2.blocks.flatMap(counted)] : block2.kind === "background" || isScenario(block2) ? [[block2.anchor, block2.hash]] : [];
  return new Map([["", feature2.hash], ...feature2.blocks.flatMap(counted)]);
}
function sameHashes(shown, other) {
  return shown.size === other.size && [...shown].every(([anchor, hash]) => other.get(anchor) === hash);
}
var LINK_ALONE = [{ slot: "link" }];
function changes(words, shown, other) {
  const counts = [
    ["other-version-changed-blocks", [...shown].filter(([anchor, hash]) => other.has(anchor) && other.get(anchor) !== hash).length],
    ["other-version-not-there", [...shown.keys()].filter((anchor) => !other.has(anchor)).length],
    ["other-version-only-there", [...other.keys()].filter((anchor) => !shown.has(anchor)).length]
  ];
  const said = counts.filter(([, count]) => count > 0).map(([id, count]) => words.text(id, { count }));
  return said.length === 0 ? LINK_ALONE : pieces(words, "other-version-changes", { changes: said.join(words.text("other-version-changes-separator")) }, ["link"]);
}
function newer(list2, version, shown) {
  const rank = (name2) => list2.precedence.findIndex((names) => name2 !== null && names.includes(name2));
  const other = rank(version);
  const own = rank(shown);
  return other >= 0 && own >= 0 && other < own;
}

// src/policies/result_groups.ts
function searchedHeading(words, text2) {
  return {
    heading: filled(words.heading, "{words}", [plainName(text2)]),
    title: words.title.replaceAll("{words}", () => text2)
  };
}
function pageStatus(words, groups) {
  const count = groups.reduce((sum, group) => sum + group.results.length, 0);
  if (count === 0) {
    return words.none;
  }
  return words.count.replaceAll("{count}", String(count)).replaceAll("{features}", String(groups.length));
}
function groupedResults(words, found2, addresses = WRITTEN_ADDRESSES) {
  const groups = /* @__PURE__ */ new Map();
  for (const one of found2) {
    const page = one.document.page;
    const group = groups.get(page) ?? { heading: featureHeading(words.headings, one.document), address: addresses.entry({ ...one.document, anchor: "" }), results: [] };
    group.results.push(listed(words, one, addresses));
    groups.set(page, group);
  }
  return [...groups.values()];
}
function listed(words, { document: document2, matched }, addresses) {
  const marks = (field) => (text2) => marked(text2, matched.get(field));
  return {
    heading: blockHeading2(words.headings, document2, marks("name")(document2.name)),
    address: addresses.entry(document2),
    within: within(words, document2),
    steps: document2.steps.map(marks("steps")).filter(holdsMatch),
    tags: document2.tags.map(marks("tags")).filter(holdsMatch),
    cells: document2.cells.map(marks("cells")).filter(holdsMatch)
  };
}
function within(words, document2) {
  const blocks = [
    document2.rule === null ? null : named2(words.headings.rule, words.headings.unnamedRule, document2.rule),
    document2.outline === null ? null : named2(words.headings.scenarioOutline, words.headings.unnamedScenarioOutline, document2.outline)
  ].filter((block2) => block2 !== null);
  if (blocks.length === 0) {
    return null;
  }
  const separator = { runs: [{ text: words.separator, marked: false }], isName: false };
  const joined = blocks.flatMap((block2, at) => at === 0 ? block2 : [separator, ...block2]);
  return filled(words.within, "{blocks}", joined);
}
function featureHeading(headings, document2) {
  return named2(headings.feature, headings.unnamedFeature, document2.feature ?? document2.name);
}
function blockHeading2(headings, document2, name2) {
  const [template, unnamed] = {
    feature: [headings.feature, headings.unnamedFeature],
    rule: [headings.rule, headings.unnamedRule],
    background: [headings.background, headings.unnamedBackground],
    scenario: [headings.scenario, headings.unnamedScenario],
    "scenario-outline": [headings.scenarioOutline, headings.unnamedScenarioOutline],
    examples: [headings.examples, headings.numberedExamples.replaceAll("{place}", String(document2.place ?? 1))]
  }[document2.kind];
  if (document2.name.trim() === "") {
    return [plainWords(unnamed)];
  }
  return filled(template, "{name}", [{ runs: name2, isName: true }]);
}
function named2(template, unnamed, name2) {
  return name2.trim() === "" ? [plainWords(unnamed)] : filled(template, "{name}", [plainName(name2)]);
}
function filled(template, placeholder, parts) {
  return template.split(placeholder).flatMap((words, at) => at === 0 ? [words] : [parts, words]).flatMap((piece) => typeof piece === "string" ? piece === "" ? [] : [plainWords(piece)] : piece);
}
function plainWords(text2) {
  return { runs: [{ text: text2, marked: false }], isName: false };
}
function plainName(text2) {
  return { runs: [{ text: text2, marked: false }], isName: true };
}

// src/policies/search_page_view.ts
function searchPageView(words, site, searched) {
  const pageWords = {
    heading: words.text("search-page-heading-for", { words: "{words}" }),
    title: words.text("search-page-title-for", { words: "{words}", site }),
    loading: words.text("search-loading"),
    failed: words.text("search-failed"),
    none: words.text("search-none"),
    count: words.text("search-page-count", { count: "{count}", features: "{features}" }),
    within: words.text("search-page-within", { blocks: "{blocks}" }),
    separator: words.text("search-page-separator"),
    matchingSteps: words.text("search-page-matching-steps"),
    matchingTags: words.text("search-page-matching-tags"),
    matchingCells: words.text("search-page-matching-cells"),
    headings: kindHeadings(words)
  };
  const text2 = searched.trim();
  return {
    title: text2 === "" ? words.text("search-page-title", { site }) : searchedHeading(pageWords, text2).title,
    heading: words.text("search-page-heading"),
    words: pageWords
  };
}
function kindHeadings(words) {
  const named3 = (kind) => words.text("named-heading", { kind, name: "{name}" });
  const unnamed = (kind) => words.text("unnamed-heading", { kind });
  return {
    feature: named3("feature"),
    unnamedFeature: unnamed("feature"),
    rule: named3("rule"),
    unnamedRule: unnamed("rule"),
    background: named3("background"),
    unnamedBackground: unnamed("background"),
    scenario: named3("scenario"),
    unnamedScenario: unnamed("scenario"),
    scenarioOutline: named3("scenario-outline"),
    unnamedScenarioOutline: unnamed("scenario-outline"),
    examples: named3("examples"),
    numberedExamples: words.text("numbered-examples", { place: "{place}" })
  };
}

// src/policies/sentences_view.ts
var KINDS = ["given", "when", "then"];
function sentencesView(words, descriptor, features) {
  const uses = usesOf(words, features);
  const groups = KINDS.filter((kind) => descriptor.sentences[kind].length > 0).map((kind) => ({
    heading: words.text(`sentences-${kind}`),
    sentences: descriptor.sentences[kind].map((text2) => {
      const found2 = uses.get(useKey(kind, text2)) ?? [];
      return { text: text2, lang: found2[0]?.lang ?? null, count: words.text("sentence-uses", { count: found2.length }), uses: found2 };
    })
  }));
  return {
    title: words.text("sentences-page-title", { site: descriptor.title }),
    heading: words.text("sentences-title"),
    groups,
    none: groups.length === 0 ? words.text("sentences-none") : null
  };
}
function usesOf(words, features) {
  const uses = /* @__PURE__ */ new Map();
  const link2 = pieces(words, "sentence-use", {}, ["heading", "path"]);
  const separator = words.text("sentence-values-separator");
  for (const { entry, object: object2 } of features) {
    const lang = object2.lang === "en" ? null : object2.lang;
    for (const block2 of blocksWithSteps(object2.feature)) {
      for (const step2 of block2.steps) {
        const use = {
          address: targetAddress({ kind: "feature", page: entry.page }, block2.anchor),
          link: link2,
          item: step2.slots.length === 0 ? null : pieces(words, "sentence-use-values", {}, ["link", "values"]),
          heading: blockHeading(words, block2),
          path: entry.path,
          values: step2.slots,
          separator,
          lang
        };
        const key = useKey(step2.kind, step2.sentence);
        const known = uses.get(key);
        if (known === void 0) {
          uses.set(key, [use]);
        } else {
          known.push(use);
        }
      }
    }
  }
  return uses;
}
function blocksWithSteps(block2) {
  return block2.blocks.flatMap((inner) => [...inner.steps.length > 0 ? [inner] : [], ...blocksWithSteps(inner)]);
}
function useKey(kind, sentence) {
  return `${kind}
${sentence}`;
}

// src/policies/tag_views.ts
function tagIndexView(words, descriptor, features) {
  return {
    title: words.text("tag-index-title"),
    entries: descriptor.tags.length === 0 ? null : descriptor.tags.map((entry) => {
      const count = features.reduce((total, feature2) => total + carriedIn(feature2.object.feature, entry.tag).length, 0);
      return { ...wordsAround(words, "tag-index-entry", { count }, {}, "tag"), address: tagAddress(entry), words: isolated(entry.tag) };
    }),
    empty: words.text("tag-index-empty")
  };
}
function tagPageView(words, descriptor, features, name2) {
  const entry = descriptor.tags.find((candidate) => candidate.page === `tags/${name2}`);
  if (entry === void 0) {
    return null;
  }
  return {
    title: words.text("tag-page-title", { tag: entry.tag }),
    heading: words.parts("tag-page-title", {}, { tag: isolated(entry.tag) }),
    outside: entry.link,
    features: features.filter((feature2) => writtenIn(feature2.object.feature, entry.tag)).map((feature2) => tagFeature2(words, feature2, entry.tag))
  };
}
function tagFeature2(words, { entry, object: object2 }, tag2) {
  const page = { kind: "feature", page: entry.page };
  const carried = carriedIn(object2.feature, tag2);
  return {
    address: targetAddress(page),
    ...wordsAround(words, "page-title", {}, { heading: heading(words, "feature", object2.feature.name) }, "path"),
    path: entry.path,
    lang: object2.lang === "en" ? null : object2.lang,
    scenarios: carried.length === 0 ? null : carried.map(({ scenario, carrier }) => {
      const link2 = { address: targetAddress(page, scenario.anchor), words: heading(words, scenario.kind, scenario.name) };
      return carrier === "itself" ? { ...link2, before: [], after: [] } : { ...link2, ...wordsAround(words, `tag-through-${carrier}`, {}, {}, "scenario") };
    }),
    none: words.text("tag-no-scenario")
  };
}
function carriedIn(feature2, tag2) {
  const onFeature = feature2.tags.includes(tag2);
  const carriers = (scenario, onRule) => {
    if (scenario.tags.includes(tag2)) {
      return "itself";
    }
    if (scenario.examples.some((examples) => examples.tags.includes(tag2))) {
      return "examples";
    }
    return onRule ? "rule" : onFeature ? "feature" : null;
  };
  return feature2.blocks.flatMap((block2) => {
    const inside = block2.kind === "rule" ? block2.blocks.filter(isScenario) : isScenario(block2) ? [block2] : [];
    const onRule = block2.kind === "rule" && block2.tags.includes(tag2);
    return inside.flatMap((scenario) => {
      const carrier = carriers(scenario, onRule);
      return carrier === null ? [] : [{ scenario, carrier }];
    });
  });
}
function writtenIn(block2, tag2) {
  return block2.tags.includes(tag2) || block2.blocks.some((inner) => writtenIn(inner, tag2)) || block2.examples.some((examples) => writtenIn(examples, tag2));
}
function tagAddress(entry) {
  return targetAddress(pageByName(entry.page));
}

// src/policies/page_frame.ts
var HOME = { kind: "home" };
function homeCrumb(siteTitle) {
  return { address: targetAddress(HOME), words: plain(siteTitle), lang: null };
}
function folderCrumbs(name2) {
  const folders = name2.split("/").slice(0, -1);
  return folders.map((folder2, place) => ({
    address: targetAddress({ kind: "folder", folder: `${folders.slice(0, place + 1).join("/")}/` }),
    words: plain(folder2),
    lang: null
  }));
}
function frameOf(siteTitle, title, lang, above, page, words) {
  return { title, lang, crumbs: [homeCrumb(siteTitle), ...above, { address: targetAddress(page), words, lang: null }] };
}
function missingFrame(words, siteTitle) {
  const heading2 = words.text("page-missing");
  return { title: words.text("page-missing-title", { heading: heading2, site: siteTitle }), lang: "en", crumbs: [homeCrumb(siteTitle)] };
}

// src/services/page_drawing.ts
function pageDrawing(parts) {
  return async (page) => {
    const descriptor = await parts.store.descriptor();
    const draw = await drawingOf(parts, descriptor, page);
    return () => {
      const frame = draw();
      if (frame === null) {
        return home(parts, descriptor);
      }
      if (descriptor.only) {
        parts.pages.onlyNote(parts.words.text("only-note"));
      }
      return frame;
    };
  };
}
function drawingOf(parts, descriptor, page) {
  switch (page.kind) {
    case "home":
      return parts.canvas.holdsFrontPage() ? () => null : folder(parts, descriptor, "");
    case "folder":
      return folder(parts, descriptor, page.folder);
    case "feature":
      return feature(parts, descriptor, page.page);
    case "tags":
      return tagIndex2(parts, descriptor);
    case "tag":
      return tag(parts, descriptor, page.tag);
    case "steps":
      return sentences2(parts, descriptor);
    case "search":
      return search(parts, descriptor, page.words);
  }
}
function home(parts, descriptor) {
  const title = parts.canvas.restoreFrontPage();
  return title === null ? missing(parts, descriptor)() : { title, lang: "en", crumbs: [homeCrumb(descriptor.title)] };
}
async function folder(parts, descriptor, name2) {
  const view = folderView(parts.words, descriptor, await loadedFeatures(parts.store, descriptor), name2);
  if (view === null) {
    return missing(parts, descriptor);
  }
  return () => {
    parts.pages.folder(view);
    const crumbs = name2 === "" ? [homeCrumb(descriptor.title)] : frameOf(descriptor.title, view.title, "en", folderCrumbs(name2.slice(0, -1)), { kind: "folder", folder: name2 }, plain(view.heading)).crumbs;
    return { title: view.title, lang: "en", crumbs };
  };
}
async function feature(parts, descriptor, page) {
  const entry = descriptor.features.find((candidate) => candidate.page === page);
  if (entry === void 0) {
    return missing(parts, descriptor);
  }
  const object2 = await parts.store.object(entry.object);
  const view = featureView(parts.words, descriptor, entry, object2);
  const versions = await inVersions(parts, descriptor, entry.page, { name: entry.object, object: object2 });
  return () => {
    parts.pages.feature(view, versions);
    return frameOf(descriptor.title, view.title, view.lang, folderCrumbs(entry.page), { kind: "feature", page }, view.heading);
  };
}
async function inVersions(parts, descriptor, page, shown) {
  if (descriptor.version === null && descriptor.preview === null) {
    return null;
  }
  try {
    const [list2, index] = await Promise.all([parts.store.versions(), parts.store.pages()]);
    const names = otherObjects(index, page, shown.name);
    const others = new Map(await Promise.all(names.map(async (name2) => [name2, await parts.store.object(name2)])));
    return pageInVersions(parts.words, descriptor, list2, index, page, shown, others);
  } catch {
    return null;
  }
}
function tagIndexCrumb(parts) {
  return { address: targetAddress({ kind: "tags" }), words: plain(parts.words.text("tag-index-title")), lang: null };
}
async function tagIndex2(parts, descriptor) {
  const view = tagIndexView(parts.words, descriptor, await loadedFeatures(parts.store, descriptor));
  return () => {
    parts.pages.tagIndex(view);
    return { title: view.title, lang: "en", crumbs: [homeCrumb(descriptor.title), tagIndexCrumb(parts)] };
  };
}
async function tag(parts, descriptor, name2) {
  const view = tagPageView(parts.words, descriptor, await loadedFeatures(parts.store, descriptor), name2);
  if (view === null) {
    return missing(parts, descriptor);
  }
  return () => {
    parts.pages.tag(view);
    return frameOf(descriptor.title, view.title, "en", [tagIndexCrumb(parts)], { kind: "tag", tag: name2 }, view.heading);
  };
}
async function sentences2(parts, descriptor) {
  const view = sentencesView(parts.words, descriptor, await loadedFeatures(parts.store, descriptor));
  return () => {
    parts.pages.sentences(view);
    return frameOf(descriptor.title, view.title, "en", [], { kind: "steps" }, plain(view.heading));
  };
}
function search(parts, descriptor, words) {
  const view = searchPageView(parts.words, descriptor.title, words);
  return () => {
    parts.pages.search(view);
    return frameOf(descriptor.title, view.title, "en", [], { kind: "search", words: "" }, plain(view.heading));
  };
}
function missing(parts, descriptor) {
  return () => {
    parts.pages.notice({ heading: parts.words.text("page-missing"), text: parts.words.text("page-missing-text") });
    return missingFrame(parts.words, descriptor.title);
  };
}

// src/services/note_explaining.ts
function startNoteExplaining(dialog) {
  let opener = null;
  dialog.onOpen((pressed) => {
    opener = pressed;
    dialog.open();
  });
  dialog.onClose(() => {
    opener?.focus();
    opener = null;
  });
}

// src/services/navigation.ts
var SETTLE_MILLISECONDS = 150;
async function startNavigation(parts) {
  const { canvas, address, timer } = parts;
  let shown = null;
  let reveal = () => {
  };
  let drawing = 0;
  let hash = address.hash();
  const focus = (id, toHeading) => {
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
  const draw = async (page, anchor, reached) => {
    drawing += 1;
    const mine = drawing;
    timer.cancel();
    if (reached) {
      canvas.focusMain();
    }
    canvas.setBusy(true);
    let drawn2;
    try {
      drawn2 = await parts.draw(page);
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
    const frame = drawn2();
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
  const show = async (target, reached) => {
    if (target.kind === "element") {
      if (drawing === 0) {
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
      address.replace(hash);
    } else {
      hash = changed;
    }
    void show(target, true);
  });
  await show(readTarget(hash), false);
}

// src/services/results_listing.ts
async function startListing(page, documents, addresses = WRITTEN_ADDRESSES) {
  const words = page.words();
  const text2 = page.searched().trim();
  if (text2 === "") {
    return;
  }
  const { heading: heading2, title } = searchedHeading(words, text2);
  page.showSearched(text2, heading2, title);
  page.setStatus(words.loading);
  let groups;
  try {
    groups = groupedResults(words, indexOf(await documents.load()).search(text2), addresses);
  } catch {
    page.setStatus(words.failed);
    return;
  }
  page.showGroups(groups);
  page.setStatus(pageStatus(words, groups));
}

// src/policies/search_results.ts
var PAUSE_MILLISECONDS = 300;
function status(words, count) {
  if (count === 0) {
    return words.none;
  }
  const template = count > words.shown ? words.countLimited : words.count;
  return template.replaceAll("{count}", String(count));
}
function result(words, document2, addresses = WRITTEN_ADDRESSES) {
  return { label: label(words, document2), address: addresses.entry(document2) };
}
function allResults(words, text2, addresses = WRITTEN_ADDRESSES) {
  const searched = text2.trim();
  if (searched === "") {
    return null;
  }
  return { label: filled2(words.allResults, { "{words}": { text: searched, isName: true } }), address: addresses.results(searched) };
}
function moved(chosen, count, step2) {
  if (count === 0) {
    return null;
  }
  if (chosen === null) {
    return step2 === 1 ? 0 : count - 1;
  }
  const next = chosen + step2;
  return next < 0 || next >= count ? null : next;
}
function label(words, document2) {
  const inRule = document2.rule !== null;
  const template = {
    feature: words.feature,
    rule: words.rule,
    background: inRule ? words.backgroundInRule : words.background,
    scenario: inRule ? words.scenarioInRule : words.scenario,
    "scenario-outline": inRule ? words.scenarioOutlineInRule : words.scenarioOutline,
    examples: inRule ? words.examplesInRule : words.examples
  }[document2.kind];
  return filled2(template, {
    "{name}": blockName(words, document2),
    "{feature}": name(words, document2.feature),
    "{rule}": name(words, document2.rule),
    "{outline}": name(words, document2.outline)
  });
}
function filled2(template, pieces2) {
  return template.split(/(\{[a-z]+\})/).filter((part2) => part2 !== "").map((part2) => pieces2[part2] ?? { text: part2, isName: false });
}
function blockName(words, document2) {
  if (document2.kind === "examples" && document2.name.trim() === "") {
    return { text: words.numberedExamples.replaceAll("{place}", String(document2.place ?? 1)), isName: false };
  }
  return name(words, document2.name);
}
function name(words, written) {
  return written === null || written.trim() === "" ? { text: words.noName, isName: false } : { text: written, isName: true };
}

// src/services/searching.ts
function startSearching(box, documents, timer, addresses = WRITTEN_ADDRESSES) {
  const words = box.words();
  let index = null;
  let loading = null;
  let text2 = box.fieldText();
  let started = false;
  let results = [];
  let chosen = null;
  let open = false;
  let pending = null;
  let paused = false;
  const load = () => {
    if (index !== null) {
      return Promise.resolve(index);
    }
    loading ??= documents.load().then((found2) => {
      index = indexOf(found2);
      return index;
    }).catch(() => {
      loading = null;
      return null;
    });
    return loading;
  };
  const announce = () => {
    if (paused && pending !== null) {
      box.setStatus(pending);
      pending = null;
    }
  };
  const show = (shown) => {
    results = shown;
    chosen = null;
    box.showResults(results);
    setOpen(results.length > 0);
  };
  const setOpen = (opened) => {
    open = opened;
    box.setOpen(opened);
  };
  const run2 = async () => {
    const asked = text2;
    if (index === null) {
      box.setStatus(words.loading);
    }
    const found2 = await load();
    if (asked !== text2) {
      return;
    }
    if (found2 === null) {
      show([]);
      pending = words.failed;
    } else {
      const matches = found2.search(asked);
      show(matches.slice(0, words.shown).map((match2) => result(words, match2.document, addresses)));
      pending = status(words, matches.length);
    }
    announce();
  };
  const empty = () => {
    timer.cancel();
    pending = null;
    show([]);
    box.setStatus("");
  };
  box.onInput((typed) => {
    text2 = typed;
    box.showAllResults(allResults(words, typed, addresses));
    if (typed.trim() === "") {
      empty();
      return;
    }
    if (!started) {
      started = true;
      void load();
    }
    paused = false;
    timer.after(PAUSE_MILLISECONDS, () => {
      paused = true;
      announce();
    });
    void run2();
  });
  const move = (step2) => {
    if (!open) {
      if (results.length === 0) {
        return;
      }
      setOpen(true);
    }
    chosen = moved(chosen, results.length, step2);
    box.choose(chosen);
  };
  box.showAllResults(allResults(words, text2, addresses));
  box.onDown(() => move(1));
  box.onUp(() => move(-1));
  box.onEnter(() => {
    const picked = open && chosen !== null ? results[chosen] : void 0;
    const target = picked ?? allResults(words, text2, addresses);
    if (target !== null) {
      box.go(target.address);
    }
  });
  box.onPick((place) => {
    const picked = results[place];
    if (picked !== void 0) {
      box.go(picked.address);
    }
  });
  box.onEscape(() => {
    if (open) {
      chosen = null;
      box.choose(null);
      setOpen(false);
      return;
    }
    text2 = "";
    box.fill("");
    box.showAllResults(null);
    empty();
  });
  return (filled3) => {
    text2 = filled3;
    box.fill(filled3);
    box.showAllResults(allResults(words, filled3, addresses));
    empty();
  };
}

// src/services/shell_behaviour.ts
function shellBehaviour(parts) {
  return (page) => {
    parts.tree.show(page);
    if (page.kind === "search") {
      parts.fillSearch?.(page.words);
      const searchPage = parts.drawn.searchPage(page.words);
      if (searchPage !== null) {
        void startListing(searchPage, parts.documents, SHELL_ADDRESSES);
      }
    }
    const backgrounds = parts.drawn.backgrounds();
    const reveal = startCollapsing(parts.drawn.blocks(), parts.memory, `${parts.shellPath}${targetAddress(page)}`, backgrounds && { backgrounds, siteRoot: parts.shellPath });
    const noteDialog = parts.drawn.noteDialog();
    if (noteDialog !== null) {
      startNoteExplaining(noteDialog);
    }
    return reveal;
  };
}

// src/policies/tree_items.ts
function treeItems(words, descriptor, features) {
  const folders = siteFolders(features);
  const itemsOf = (folder2) => folder2 === void 0 ? [] : [
    ...folder2.folders.map((inner) => ({ text: folderName(inner), address: targetAddress({ kind: "folder", folder: inner }), items: itemsOf(folders.get(inner)) })),
    ...folder2.features.map(({ entry, object: object2 }) => ({
      text: runsOf(heading(words, "feature", object2.feature.name)),
      address: targetAddress({ kind: "feature", page: entry.page }),
      lang: object2.lang,
      items: contentsOf(words, entry.page, object2.feature).map(contentsItem)
    }))
  ];
  return [{ text: descriptor.title, address: targetAddress({ kind: "home" }) }, ...itemsOf(folders.get(""))];
}
function pathToPage(items, page) {
  const address = targetAddress(page);
  const pathTo2 = (inside) => {
    for (const item of inside) {
      if (item.address === address) {
        return [item.address];
      }
      const inner = pathTo2(item.items ?? []);
      if (inner.length > 0) {
        return [...inner, item.address];
      }
    }
    return [];
  };
  return pathTo2(items);
}
function contentsItem(entry) {
  return { text: runsOf(entry.heading), address: entry.address, items: entry.entries.map(contentsItem) };
}
function runsOf(parts) {
  const runs = parts.reduce((done, part2) => {
    const last2 = done.length - 1;
    if (part2.isolated === (last2 % 2 === 1)) {
      return [...done.slice(0, last2), `${done[last2] ?? ""}${part2.text}`];
    }
    return [...done, part2.text];
  }, [""]);
  return runs.length === 1 ? runs[0] ?? "" : runs;
}

// src/services/shell_tree.ts
function startShellTree(parts) {
  const controls = startTreeControls(parts.toggle, parts.width, parts.memory, parts.sitePath);
  let items = null;
  let current = null;
  let usable = true;
  const build = () => {
    if (usable && items !== null && current !== null) {
      buildTree(parts.view, parts.memory, parts.sitePath, items, pathToPage(items, current));
    }
  };
  const disable = () => {
    if (usable) {
      usable = false;
      controls.disable();
    }
  };
  void (async () => {
    try {
      const descriptor = await parts.store.descriptor();
      items = treeItems(parts.words, descriptor, await loadedFeatures(parts.store, descriptor));
      build();
    } catch {
      disable();
    }
  })();
  return {
    show: (page) => {
      current = page;
      build();
    },
    disable
  };
}

// src/policies/theme_choice.ts
var THEME_KEY = "gherkin-glass:theme";
function keptChoice(kept) {
  return kept === "light" || kept === "dark" ? kept : "system";
}
function themeAttribute(choice) {
  return choice === "system" ? null : choice;
}

// src/services/theme_switching.ts
function startThemeSwitching(theme, memory) {
  const showKept = () => {
    const choice = keptChoice(memory.get(THEME_KEY));
    theme.show(choice);
    theme.apply(themeAttribute(choice));
  };
  showKept();
  theme.onRestore(showKept);
  theme.onChoose((chosen) => {
    theme.apply(themeAttribute(chosen));
    memory.set(THEME_KEY, chosen);
  });
}

// src/main.ts
var descriptorPath = document.documentElement.getAttribute("data-ghg-descriptor");
if (descriptorPath === null) {
  startWrittenPage();
} else {
  startShell(descriptorPath);
}
function startWrittenPage() {
  const memory = new LocalMemory();
  startThemeSwitching(new DomTheme(document), memory);
  const backgrounds = DomBackgrounds.of(document);
  startCollapsing(
    new DomPage(document),
    memory,
    document.location.pathname,
    backgrounds === null ? null : { backgrounds, siteRoot: siteRoot(import.meta.url) }
  );
  const noteDialog = DomNoteDialog.of(document);
  if (noteDialog !== null) {
    startNoteExplaining(noteDialog);
  }
  const site = new URL("../", import.meta.url);
  const sitePath = site.pathname;
  const page = pageAddress(sitePath, document.location.pathname);
  void startFolderTree(
    new FetchedTree(new URL("tree.json", import.meta.url)),
    new DomTreeSwitch(document),
    new DomTree(document, site, page),
    new MediaWidth(window.matchMedia(NARROW)),
    memory,
    { sitePath, page }
  );
  const searchDocuments2 = new FetchedDocuments(new URL("search.json", import.meta.url));
  const searchPage = DomSearchPage.find(document);
  if (searchPage !== null) {
    void startListing(searchPage, searchDocuments2);
  }
  const searchBox = DomSearchBox.find(document);
  if (searchBox !== null) {
    startSearching(searchBox, searchDocuments2, new WindowTimer());
  }
}
function startShell(descriptorPath2) {
  const make = new ElementBuilder(document);
  const canvas = DomCanvas.of(make);
  if (canvas === null) {
    return;
  }
  const memory = new LocalMemory();
  startThemeSwitching(new DomTheme(document), memory);
  const root = new URL("../../", import.meta.url);
  const words = new FluentWords(document.getElementById("ghg_messages")?.textContent ?? "");
  const store = new FetchedStore(new URL("data/", root), descriptorPath2);
  const shellPath = document.location.pathname.replace(/index\.html$/, "");
  const tree = startShellTree({
    store,
    words,
    toggle: new DomTreeSwitch(document),
    view: DomTree.inShell(document),
    width: new MediaWidth(window.matchMedia(NARROW)),
    memory,
    sitePath: shellPath
  });
  const documents = storeDocuments(store);
  const searchBox = DomSearchBox.find(document);
  const fillSearch = searchBox === null ? null : startSearching(searchBox, documents, new WindowTimer(), SHELL_ADDRESSES);
  void startNavigation({
    canvas,
    address: new DomAddress(window),
    timer: new WindowTimer(),
    draw: pageDrawing({ store, words, pages: new DomPages(make, canvas, words), canvas }),
    behave: shellBehaviour({ drawn: new DomDrawnPage(document, canvas), tree, fillSearch, documents, memory, shellPath }),
    failure: words.text("page-not-loaded"),
    onFailure: () => tree.disable()
  });
}
