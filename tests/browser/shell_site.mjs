// A site of the form of gherkin-glass 2.0, written for the browser checks
// while ghg still writes every page whole: one shell, made from the layout of
// a page ghg wrote, tests/fixtures/pages/expected/billing/refunds.html, the
// script and the stylesheet of assets/, and a store holding the data of the
// script's own tests, script/test/support/, served over HTTP from a
// temporary folder. The shell may hold a front page, and may be a version's,
// with an older version of the refunds' page in the store.

import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { extname, join, normalize, resolve } from "node:path";

import { billingFeatures } from "../../script/test/support/billing_store.ts";
import { outlinesEntry, outlinesObject } from "../../script/test/support/outlines_store.ts";
import { refundsTags } from "../../script/test/support/refunds_store.ts";
import { descriptor } from "../../script/test/support/store_building.ts";

const ASSETS = "assets/test";
const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".css": "text/css", ".svg": "image/svg+xml", ".json": "application/json" };

/** The features of the shell, each with its object. */
export const FEATURES = [...billingFeatures, { entry: outlinesEntry, object: outlinesObject }];

/** The shell's layout, its `main` holding `frontPage`, its breadcrumb and tree emptied, its links to the shared assets and its words for the script. */
function shell(frontPage) {
  const layout = readFileSync(resolve("tests/fixtures/pages/expected/billing/refunds.html"), "utf8");
  const messages = readFileSync(resolve("messages/en/gherkin-glass.ftl"), "utf8");
  return layout
    .replace('<html lang="en">', '<html lang="en" data-ghg-descriptor="site.json">')
    .replace(/<title>[^<]*<\/title>/, "<title>Banking &#38; &#60;more&#62;</title>")
    .replaceAll("../assets/gherkin-glass.", `${ASSETS}/gherkin-glass.`)
    .replace(/<nav class="ghg-breadcrumb" aria-label="Breadcrumb">[\s\S]*?<\/nav>/, '<nav class="ghg-breadcrumb" aria-label="Breadcrumb"></nav>')
    .replace(/(<nav id="ghg_tree"[^>]*>)[\s\S]*?(<\/nav>)/, "$1$2")
    .replace('href="#ghg_main"', 'href="##ghg_main"')
    .replace('data-root="../"', 'data-root=""')
    .replace('href="../search.html"', 'href="#search"')
    .replace(/<main id="ghg_main">[\s\S]*<\/main>/, `<main id="ghg_main" tabindex="-1" aria-busy="true">${frontPage}</main>\n<p id="ghg_announcement" role="status"></p>\n<script type="text/plain" id="ghg_messages">${messages}</script>`);
}

/** The store of a version, v1.0.0, whose refunds' page in `main` has another feature's hash and its first scenario changed. */
function versionFiles(folder) {
  const refunds = FEATURES.find((feature) => feature.entry.page === "billing/refunds");
  const older = structuredClone(refunds.object);
  older.feature.hash = "changed";
  const changed = older.feature.blocks.find((block) => block.kind === "scenario");
  if (changed !== undefined) changed.hash = "changed";
  writeFileSync(join(folder, "data/objects/refunds-main.json"), JSON.stringify(older));
  writeFileSync(join(folder, "data/versions.json"), JSON.stringify({ format: 1, development: ["main"], releases: ["v1.0.0"], pre_releases: [], precedence: [["v1.0.0"]] }));
  const objects = [
    { object: refunds.entry.object, versions: ["v1.0.0"] },
    { object: "refunds-main", versions: ["main"] },
  ];
  writeFileSync(join(folder, "data/pages.json"), JSON.stringify({ format: 1, pages: [{ page: "billing/refunds", objects }] }));
}

/** Writes the site into a new temporary folder, serves it, and resolves with its address and a way to stop. */
export async function serveShellSite({ frontPage = "", versioned = false } = {}) {
  const folder = mkdtempSync(join(tmpdir(), "ghg-shell-"));
  mkdirSync(join(folder, ASSETS), { recursive: true });
  mkdirSync(join(folder, "data/objects"), { recursive: true });
  for (const file of ["gherkin-glass.js", "gherkin-glass.css", "gherkin-glass.svg"]) {
    copyFileSync(resolve("assets", file), join(folder, ASSETS, file));
  }
  writeFileSync(join(folder, "index.html"), shell(frontPage));
  const site = descriptor(
    "Banking & <more>",
    FEATURES.map((feature) => feature.entry),
    { tags: refundsTags, version: versioned ? "v1.0.0" : null },
  );
  if (versioned) {
    versionFiles(folder);
  }
  writeFileSync(join(folder, "data/site.json"), JSON.stringify(site));
  for (const { entry, object } of FEATURES) {
    writeFileSync(join(folder, "data/objects", `${entry.object}.json`), JSON.stringify(object));
  }
  const server = createServer((request, response) => {
    const path = normalize(decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname)).replace(/^[/\\]+/, "");
    try {
      const body = readFileSync(join(folder, path === "" ? "index.html" : path));
      response.writeHead(200, { "content-type": TYPES[extname(path)] ?? "text/html; charset=utf-8" });
      response.end(body);
    } catch {
      response.writeHead(404, { "content-type": "text/plain" });
      response.end("Nothing is at this address.");
    }
  });
  await new Promise((done) => server.listen(0, "127.0.0.1", done));
  const address = `http://127.0.0.1:${server.address().port}/index.html`;
  return {
    address,
    stop: async () => {
      await new Promise((done) => server.close(done));
      rmSync(folder, { recursive: true, force: true });
    },
  };
}
