// Spike: every page of a version drawn from shared data, for gherkin-glass issue #173.
// Not ghg's script. It covers home, folder and feature pages, both address forms, and the
// versions a block is the same or changed in, from the blocks' hashes.

const shell = document.documentElement;
const VERSION = shell.dataset.version;
const ROUTING = shell.dataset.routing; // "hash" or "query"
const main = document.getElementById("ghg_main");
const crumbs = document.getElementById("ghg_breadcrumb");
const announcer = document.getElementById("ghg_announcer");
const KIND = { feature: "Feature", background: "Background", rule: "Rule", scenario: "Scenario", "scenario-outline": "Scenario outline", examples: "Examples" };
const KIND_WORD = { feature: ["Feature"], background: ["Background"], rule: ["Rule"], scenario: ["Scenario"], "scenario-outline": ["Scenario Outline"], examples: ["Examples"] };
const NOTE_KIND = { feature: "this feature is", background: "this background is", rule: "this rule is", scenario: "this scenario is", "scenario-outline": "this scenario outline is", examples: "these examples are" };

// ---- Elements, always with text as text.
function h(tag, attrs, ...children) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs ?? {})) {
    if (v === null || v === undefined || v === false) continue;
    if (k === "hidden") e.hidden = true; else e.setAttribute(k, v === true ? "" : v);
  }
  for (const c of children.flat()) if (c !== null && c !== undefined && c !== false) e.append(c);
  return e;
}
const fill = (parent, ...children) => parent.replaceChildren(...children.flat(2).filter((c) => c !== null && c !== undefined && c !== false));
const bdi = (text) => h("bdi", null, text);
const heading = (kind, name) => name === "" ? [KIND[kind]] : [`${KIND[kind]}: `, bdi(name)];
const headingText = (kind, name) => name === "" ? KIND[kind] : `${KIND[kind]}: ${name}`;

// ---- Addresses.
const encodePath = (p) => p.split("/").map(encodeURIComponent).join("/");
function readAddress() {
  if (ROUTING === "hash") {
    const raw = location.hash.slice(1);
    if (raw.startsWith("#")) return { own: decodeURIComponent(raw.slice(1)) };
    const at = raw.indexOf("#");
    const page = decodeURIComponent(at < 0 ? raw : raw.slice(0, at));
    const anchor = at < 0 ? "" : decodeURIComponent(raw.slice(at + 1));
    return { page, anchor };
  }
  const page = new URLSearchParams(location.search).get("page") ?? "";
  return { page, anchor: decodeURIComponent(location.hash.slice(1)) };
}
function address(page, anchor = "", version = VERSION) {
  if (ROUTING === "query" && version === VERSION && page === current && anchor) return `#${encodeURIComponent(anchor)}`;
  const base = version === VERSION ? "" : `../${encodeURIComponent(version)}/index.html`;
  if (ROUTING === "hash") return `${base}#${encodePath(page)}${anchor ? "#" + encodeURIComponent(anchor) : ""}`;
  return `${base || "index.html"}${page ? "?page=" + encodePath(page) : ""}${anchor ? "#" + encodeURIComponent(anchor) : ""}`;
}
let current = null;
const ownTarget = (id) => (ROUTING === "hash" ? "##" : "#") + id;

// ---- Data.
const fetchJson = async (path) => (await fetch(new URL(path, import.meta.url))).json();
const [versionsList, store] = await Promise.all([fetchJson("data/versions.json"), fetchJson("data/store.json")]);
const manifests = Object.fromEntries(await Promise.all(versionsList.versions.map(async (v) => [v, await fetchJson(`data/versions/${v}.json`)])));
const manifest = manifests[VERSION];
const TITLE = manifest.title;

// For each version, each page's blocks by anchor, with their hashes.
function blocksByAnchor(m) {
  const pages = new Map();
  for (const f of m.files) {
    const byAnchor = new Map();
    const walk = (items) => { for (const i of items) { byAnchor.set(i.anchor, i.block); if (i.children) walk(i.children); } };
    walk(f.children);
    pages.set(f.page, { feature: f.feature, byAnchor });
  }
  return pages;
}
const index = Object.fromEntries(Object.entries(manifests).map(([v, m]) => [v, blocksByAnchor(m)]));

// ---- The folders of the version.
function folderOf(page) { const i = page.lastIndexOf("/"); return i < 0 ? "" : page.slice(0, i + 1); }
function folderContents(folder) {
  const subfolders = new Set(); const features = [];
  for (const f of manifest.files) {
    if (!f.page.startsWith(folder)) continue;
    const rest = f.page.slice(folder.length);
    const slash = rest.indexOf("/");
    if (slash < 0) features.push(f); else subfolders.add(rest.slice(0, slash + 1));
  }
  return { subfolders: [...subfolders].sort(), features };
}

// ---- Steps, highlighted as today's pages highlight them.
function highlight(text, columns) {
  const out = []; let plain = "";
  const flush = () => { if (plain) { out.push(...numbers(plain)); plain = ""; } };
  for (let i = 0; i < text.length;) {
    if (text[i] === '"' && text[i - 1] !== "\\") {
      const end = text.indexOf('"', i + 1);
      if (end > i) { flush(); out.push(h("span", { class: "ghg-parameter" }, ...placeholders(text.slice(i, end + 1), columns))); i = end + 1; continue; }
    }
    const ph = columns && text[i] === "<" ? text.slice(i).match(/^<([^>]+)>/) : null;
    if (ph && columns.includes(ph[1])) { flush(); out.push(h("span", { class: "ghg-placeholder" }, ph[0])); i += ph[0].length; continue; }
    plain += text[i]; i++;
  }
  flush(); return out;
}
function placeholders(text, columns) {
  if (!columns) return [text];
  const out = []; let last = 0;
  for (const m of text.matchAll(/<([^>]+)>/g)) {
    if (!columns.includes(m[1])) continue;
    out.push(text.slice(last, m.index), h("span", { class: "ghg-placeholder" }, m[0])); last = m.index + m[0].length;
  }
  out.push(text.slice(last)); return out.filter((x) => x !== "");
}
function numbers(text) {
  const out = []; let last = 0;
  for (const m of text.matchAll(/(?<![\p{L}\p{N}])\d+(?:[.,]\d+)?(?![\p{L}\p{N}])/gu)) {
    out.push(text.slice(last, m.index), h("span", { class: "ghg-number" }, m[0])); last = m.index + m[0].length;
  }
  out.push(text.slice(last)); return out.filter((x) => x !== "");
}
function substitute(text, row) { return text.replace(/<([^>]+)>/g, (all, name) => (name in row ? row[name] : all)); }
function table(rows, sub) {
  const [head, ...body] = rows;
  return h("table", { class: "ghg-table" },
    h("thead", null, h("tr", null, head.map((c) => h("th", { scope: "col" }, sub(c))))),
    body.length ? h("tbody", null, body.map((r) => h("tr", null, r.map((c) => h("td", null, sub(c)))))) : null);
}
function steps(list, { columns = null, row = null } = {}) {
  const sub = row ? (t) => substitute(t, row) : (t) => t;
  return h("ol", { class: "ghg-steps" }, list.map((s) => h("li", null,
    h("span", { class: "ghg-keyword" }, s.keyword), ...highlight(sub(s.text), row ? null : columns),
    s.table ? ["\n", table(s.table, sub)] : null,
    s.docString ? ["\n", h("pre", { class: "ghg-doc-string" }, sub(s.docString.content))] : null)));
}
function tagList(tags) { return tags && tags.length ? h("ul", { class: "ghg-tags" }, tags.map((t) => h("li", null, t))) : null; }
function description(text) {
  if (!text) return null;
  const lines = text.split("\n"); const p = h("p");
  lines.forEach((l, i) => { if (i) p.append(h("br"), "\n"); p.append(l); });
  return h("div", { class: "ghg-description" }, p);
}
function keywordNote(kind, keyword) {
  const words = KIND_WORD[kind];
  if (words.some((w) => w.toLowerCase() === keyword.toLowerCase())) return null;
  return h("p", { class: "ghg-keyword-note", role: "note" }, `Note: in the Gherkin feature file, ${NOTE_KIND[kind]} defined with the keyword "`, bdi(keyword), `".`);
}

// ---- What other versions say of a block, from its hash.
function versionsOf(page, anchor, hash) {
  const items = [];
  for (const v of versionsList.versions) {
    if (v === VERSION) continue;
    const other = index[v].get(page);
    const theirs = other?.byAnchor.get(anchor);
    if (theirs === undefined) items.push(h("li", null, h("a", { href: address("", "", v) }, v), ", not in this version"));
    else items.push(h("li", null, h("a", { href: address(page, anchor, v) }, v), theirs === hash ? ", unchanged" : ", changed"));
  }
  return items.length ? h("div", { class: "ghg-spike-versions" }, h("p", null, "In other versions:"), h("ul", null, items)) : null;
}
function pageVersions(file) {
  const mine = index[VERSION].get(file.page);
  const items = versionsList.versions.filter((v) => v !== VERSION).map((v) => {
    const other = index[v].get(file.page);
    if (!other) return h("li", null, h("a", { href: address("", "", v) }, v), ", which does not have this feature");
    let changed = 0, missing = 0;
    for (const [a, hash] of mine.byAnchor) { const t = other.byAnchor.get(a); if (t === undefined) missing++; else if (t !== hash) changed++; }
    const onlyThere = [...other.byAnchor.keys()].filter((a) => !mine.byAnchor.has(a)).length;
    const parts = [];
    if (changed) parts.push(`${changed} changed`);
    if (missing) parts.push(`${missing} not there`);
    if (onlyThere) parts.push(`${onlyThere} only there`);
    if (other.feature !== mine.feature) parts.push("the feature's own text changed");
    return h("li", null, h("a", { href: address(file.page, "", v) }, v), parts.length ? `: ${parts.join(", ")}` : ": the same");
  });
  return h("nav", { class: "ghg-spike-page-versions", "aria-label": "This page in other versions" },
    h("p", null, "This page in other versions:"), h("ul", null, items));
}

// ---- Blocks.
function block(cls, level, id, label, expanded, content, item) {
  return h("div", { class: `ghg-block ${cls}` },
    h(`h${level}`, { id }, h("button", { type: "button", class: "ghg-toggle", "aria-expanded": String(expanded) }, ...label)),
    h("div", { class: "ghg-content", hidden: !expanded }, ...content));
}
function scenario(file, item, level) {
  const b = store[item.block];
  const outline = b.kind === "scenario-outline";
  const columns = outline ? [...new Set(b.examples.flatMap((x) => (x.table ? x.table[0] : [])))] : null;
  const content = [keywordNote(b.kind, b.keyword), versionsOf(file.page, item.anchor, item.block), tagList(b.tags), description(b.description), steps(b.steps, { columns })];
  if (outline) {
    let row = 0;
    const hasRows = b.examples.some((x) => x.table && x.table.length > 1);
    if (hasRows) content.push(h("p", { id: `${item.anchor}_scenarios`, class: "ghg-runs" }, "Runs once for each row of these examples:"));
    content.push(h("ul", { class: "ghg-examples-list" }, b.examples.map((x, place) => {
      const anchor = item.examples[place];
      const rows = x.table ? x.table.slice(1) : [];
      const label = x.name === "" ? [`Examples ${place + 1}`] : ["Examples: ", bdi(x.name)];
      if (!rows.length) label.push(" (nothing to run)");
      const runs = rows.map((cells) => {
        row++;
        const values = Object.fromEntries(x.table[0].map((c, i) => [c, cells[i]]));
        const title = [bdi(b.name || "Scenario outline"), " ("];
        x.table[0].forEach((c, i) => { if (i) title.push(", "); title.push(bdi(c), ": ", bdi(cells[i])); });
        title.push(")");
        return block("ghg-scenario", level + 2, `${item.anchor}-row-${row}`, title, true, [steps(b.steps, { row: values })]);
      });
      return h("li", { id: anchor, class: "ghg-block ghg-examples" },
        h("button", { type: "button", class: "ghg-toggle", "aria-expanded": "false" }, ...label),
        h("div", { class: "ghg-content", hidden: true }, keywordNote("examples", x.keyword), tagList(x.tags), description(x.description),
          x.table ? table(x.table, (t) => t) : null,
          runs.length ? block("ghg-examples-as-scenarios", level + 1, `${anchor}_runs`, ["Runs, one per row"], false, runs) : null));
    })));
  }
  return block("ghg-scenario", level, item.anchor, heading(b.kind, b.name), true, content);
}
function background(file, item, level) {
  const b = store[item.block];
  return block("ghg-background", level, item.anchor, heading("background", b.name), true,
    [keywordNote("background", b.keyword), versionsOf(file.page, item.anchor, item.block), description(b.description), steps(b.steps)]);
}
function child(file, item, level) {
  const kind = store[item.block].kind;
  if (kind === "background") return background(file, item, level);
  if (kind === "rule") {
    const b = store[item.block];
    return block("ghg-rule", level, item.anchor, heading("rule", b.name), true,
      [keywordNote("rule", b.keyword), versionsOf(file.page, item.anchor, item.block), tagList(b.tags), description(b.description),
        ...item.children.map((c) => child(file, c, level + 1))]);
  }
  return scenario(file, item, level);
}
function contents(file) {
  const link = (i) => { const b = store[i.block]; return h("a", { href: address(file.page, i.anchor) }, ...heading(b.kind, b.name)); };
  const listed = (items) => items.filter((i) => store[i.block].kind !== "background");
  const hasRule = file.children.some((i) => store[i.block].kind === "rule");
  const outside = listed(file.children).filter((i) => store[i.block].kind !== "rule");
  const items = [];
  if (hasRule && outside.length) items.push(h("li", null, h("a", { href: address(file.page, "outside_rules") }, "Outside any rule"), h("ul", null, outside.map((i) => h("li", null, link(i))))));
  for (const i of file.children) {
    const kind = store[i.block].kind;
    if (kind === "rule") items.push(h("li", null, link(i), h("ul", null, listed(i.children).map((c) => h("li", null, link(c))))));
    else if (!hasRule && kind !== "background") items.push(h("li", null, link(i)));
  }
  return items.length ? h("nav", { class: "ghg-contents", "aria-label": "Contents" }, h("ul", null, items)) : null;
}

// ---- Pages.
function breadcrumb(page, last) {
  const items = [h("li", null, h("a", { href: "../index.html" }, "Versions")), h("li", null, h("a", { href: address("") , "aria-current": page === "" ? "page" : null }, VERSION))];
  const parts = page.split("/").filter(Boolean);
  let path = "";
  parts.forEach((part, i) => {
    const isLast = i === parts.length - 1;
    path += part + (isLast && !page.endsWith("/") ? "" : "/");
    items.push(h("li", null, h("a", { href: address(path), "aria-current": isLast ? "page" : null }, ...(isLast && last ? last : [part]))));
  });
  crumbs.replaceChildren(h("ol", null, items));
}
function folderPage(folder) {
  const { subfolders, features } = folderContents(folder);
  const name = folder === "" ? TITLE : folder.slice(0, -1).split("/").pop();
  breadcrumb(folder);
  const scenarios = (f) => { let n = 0; const walk = (items) => items.forEach((i) => { const k = store[i.block].kind; if (k === "rule") walk(i.children); else if (k !== "background") n++; }); walk(f.children); return n; };
  fill(main, h("h1", { tabindex: "-1" }, name),
    subfolders.length ? [h("h2", null, "Folders"), h("ul", { class: "ghg-folders" }, subfolders.map((s) => h("li", null, h("a", { href: address(folder + s) }, s.slice(0, -1)))))] : null,
    features.length ? [h("h2", null, "Features"), h("ul", { class: "ghg-features" }, features.map((f) => {
      const n = scenarios(f);
      return h("li", { class: "ghg-feature" }, h("a", { href: address(f.page) }, "Feature: ", h("bdi", { lang: store[f.feature].language }, store[f.feature].name)), ` (${n} ${n === 1 ? "scenario" : "scenarios"})`);
    }))] : null);
  return folder === "" ? `${TITLE}, ${VERSION}` : `${name}, ${TITLE}, ${VERSION}`;
}
function featurePage(file) {
  const f = store[file.feature];
  breadcrumb(file.page, heading("feature", f.name));
  const hasRule = file.children.some((i) => store[i.block].kind === "rule");
  const before = []; const outside = []; const rules = [];
  for (const i of file.children) {
    const kind = store[i.block].kind;
    if (kind === "background") before.push(background(file, i, 2));
    else if (kind === "rule") rules.push(child(file, i, 2));
    else outside.push(scenario(file, i, hasRule ? 3 : 2));
  }
  main.lang = f.language;
  fill(main, h("h1", { tabindex: "-1" }, ...heading("feature", f.name)),
    keywordNote("feature", f.keyword), pageVersions(file), tagList(f.tags), description(f.description), contents(file),
    h("div", { class: "ghg-controls" }, h("button", { type: "button", class: "ghg-collapse-all" }, "Collapse all"), h("button", { type: "button", class: "ghg-expand-all" }, "Expand all")),
    ...before, hasRule && outside.length ? block("ghg-outside-rules", 2, "outside_rules", ["Outside any rule"], true, outside) : outside, ...rules);
  return `${headingText("feature", f.name)}, ${file.path}, ${VERSION}`;
}
function notFound(page) {
  breadcrumb("");
  fill(main, h("h1", { tabindex: "-1" }, "Nothing is at this address"), h("p", null, `${VERSION} has no page `, h("code", null, page), ". ", h("a", { href: address("") }, `Go to ${VERSION}'s home page`), "."));
  return `Nothing is at this address, ${VERSION}`;
}

// ---- Drawing a page, and moving to an anchor.
function draw(page) {
  main.setAttribute("aria-busy", "true");
  main.removeAttribute("lang");
  current = page;
  const file = manifest.files.find((f) => f.page === page);
  const title = page === "" || page.endsWith("/") ? folderPage(page) : file ? featurePage(file) : notFound(page);
  document.title = title;
  main.removeAttribute("aria-busy");
  current = page;
  return title;
}
function reveal(id) {
  const target = document.getElementById(id);
  if (!target) return null;
  for (let e = target; e && e !== main; e = e.parentElement) {
    if (e.classList.contains("ghg-content")) { e.hidden = false; toggleOf(e)?.setAttribute("aria-expanded", "true"); }
  }
  const own = target.matches("li.ghg-examples") ? target.querySelector(":scope > .ghg-content") : target.matches("h1,h2,h3,h4,h5,h6") ? target.nextElementSibling : null;
  if (own?.classList.contains("ghg-content")) { own.hidden = false; toggleOf(own)?.setAttribute("aria-expanded", "true"); }
  return target;
}
function focusOn(target) {
  if (!target.hasAttribute("tabindex")) target.setAttribute("tabindex", "-1");
  target.scrollIntoView();
  target.focus({ preventScroll: true });
}
function toggleOf(content) {
  const before = content.previousElementSibling;
  if (!before) return null;
  return before.matches("button.ghg-toggle") ? before : before.querySelector(":scope > button.ghg-toggle");
}
function announce(text) {
  announcer.textContent = "";
  setTimeout(() => { announcer.textContent = text; }, 50);
}

// ---- Behaviour: collapsing, and the theme.
document.addEventListener("click", (event) => {
  const button = event.target.closest("button");
  if (!button) return;
  if (button.matches(".ghg-toggle")) {
    const content = button.parentElement.matches("li.ghg-examples") ? button.nextElementSibling : button.parentElement.nextElementSibling;
    const open = button.getAttribute("aria-expanded") !== "true";
    button.setAttribute("aria-expanded", String(open)); content.hidden = !open;
  } else if (button.matches(".ghg-collapse-all, .ghg-expand-all")) {
    const open = button.matches(".ghg-expand-all");
    for (const t of main.querySelectorAll(".ghg-toggle")) {
      t.setAttribute("aria-expanded", String(open));
      (t.parentElement.matches("li.ghg-examples") ? t.nextElementSibling : t.parentElement.nextElementSibling).hidden = !open;
    }
  }
});
for (const radio of document.querySelectorAll('input[name="ghg-theme"]')) {
  try { if ((localStorage.getItem("gherkin-glass:theme") ?? "system") === radio.value) radio.checked = true; } catch {}
  radio.addEventListener("change", () => {
    if (radio.value === "system") delete shell.dataset.theme; else shell.dataset.theme = radio.value;
    try { localStorage.setItem("gherkin-glass:theme", radio.value); } catch {}
  });
}
document.querySelector(".ghg-skip").setAttribute("href", ownTarget("ghg_main"));

// ---- Following the address.
document.addEventListener("click", (event) => {
  const link = event.target.closest('a[href^="##"]');
  if (!link) return;
  event.preventDefault();
  const t = document.getElementById(link.getAttribute("href").slice(2));
  if (t) focusOn(t);
});
function follow(first) {
  const at = readAddress();
  if (at.own !== undefined) {
    if (current === null) draw("");
    const t = document.getElementById(at.own); if (t) focusOn(t); return;
  }
  let title = null;
  // The link followed is about to be removed with the page it is on: keep the focus in main
  // meanwhile, so that it never falls back to the body and a screen reader to the page's top.
  if (at.page !== current) { if (!first) main.focus({ preventScroll: true }); title = draw(at.page); }
  const finish = () => {
    const target = at.anchor ? reveal(at.anchor) : null;
    if (target) focusOn(target);
    else if (!first && title) { window.scrollTo(0, 0); focusOn(main.querySelector("h1")); }
    if (!first && title) announce(title);
  };
  // A screen reader rebuilds its view of the page after the page changes; moving the focus
  // before it has done so can leave its cursor at the page's top.
  if (first || title === null) finish(); else setTimeout(() => requestAnimationFrame(finish), 150);
}
if (ROUTING === "hash") window.addEventListener("hashchange", () => follow(false));
else window.addEventListener("hashchange", () => { const t = reveal(readAddress().anchor); if (t) focusOn(t); });
follow(true);
