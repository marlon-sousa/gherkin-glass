// Tool: draws every page of every shell of the site in a folder with the site's own script, in happy-dom, and writes each page's HTML below another folder, as CONTRIBUTING.md, "Fixtures", names them.

import { mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative, sep } from "node:path";

import { Browser } from "happy-dom";

import { targetAddress, type PageName } from "../src/policies/address.ts";
import { shellPages, type PagesDescriptor } from "./shell_pages.ts";

const ORIGIN = "https://site.test/";

/** The folder of the assets the committed sites name with the placeholder, `{version}`, as an address writes it. */
const PLACEHOLDER_ASSETS = "assets/%7Bversion%7D/";

const [site, drawn] = process.argv.slice(2);
if (site === undefined || drawn === undefined) {
  throw new Error("Usage: node tools/drawn_pages.ts <site folder> <drawn folder>");
}

const browser = new Browser({
  settings: {
    enableJavaScriptEvaluation: true,
    suppressInsecureJavaScriptEnvironmentWarning: true,
    fetch: {
      virtualServers: [
        // happy-dom 21.0.4 looks a request's path up as it is, still percent-encoded.
        { url: `${ORIGIN}${PLACEHOLDER_ASSETS}`, directory: join(site, "assets", "{version}") },
        { url: ORIGIN, directory: site },
      ],
    },
  },
});

rmSync(drawn, { recursive: true, force: true });
try {
  for (const shell of shells(site)) {
    const descriptor = JSON.parse(readFileSync(join(site, "data", shell.descriptor), "utf8")) as PagesDescriptor;
    for (const page of shellPages(descriptor)) {
      const file = join(drawn, shell.name, pageFile(page));
      mkdirSync(dirname(file), { recursive: true });
      writeFileSync(file, await drawnPage(`${shell.path}${targetAddress(page)}`));
    }
  }
} finally {
  await browser.close();
}

/** A shell of the site: its file, from the site's folder, the descriptor it names, and its folder among the drawn pages. */
interface Shell {
  readonly path: string;
  readonly descriptor: string;
  readonly name: string;
}

/** The shells below `folder`: every page of HTML that names a descriptor. */
function shells(folder: string): Shell[] {
  return readdirSync(folder, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name === "index.html")
    .map((entry) => relative(folder, join(entry.parentPath, entry.name)).split(sep).join("/"))
    .sort()
    .flatMap((path) => {
      const descriptor = /<html[^>]* data-ghg-descriptor="([^"]+)"/.exec(readFileSync(join(folder, path), "utf8"))?.[1];
      const name = path === "index.html" ? "site" : path.slice(0, -"/index.html".length);
      return descriptor === undefined ? [] : [{ path, descriptor, name }];
    });
}

/** The file of `page` among its shell's drawn pages, one folder for each kind, so that no two clash. */
function pageFile(page: PageName): string {
  switch (page.kind) {
    case "home":
      return "folder/index.html";
    case "folder":
      return `folder/${page.folder}index.html`;
    case "feature":
      return `feature/${page.page}.html`;
    case "tags":
      return "tags.html";
    case "tag":
      return `tag/${page.tag}.html`;
    case "steps":
      return "steps.html";
    case "search":
      return "search.html";
  }
}

/** The document the shell at `address`, from the site's folder, draws, once its `main` is no longer busy, in a window of its own. */
async function drawnPage(address: string): Promise<string> {
  const page = browser.newPage();
  await page.goto(`${ORIGIN}${address}`);
  await page.waitUntilComplete();
  const document = page.mainFrame.document;
  if (document.getElementById("ghg_main")?.hasAttribute("aria-busy") !== false) {
    throw new Error(`${address} was never drawn:\n${page.virtualConsolePrinter.readAsString()}`);
  }
  const html = `<!doctype html>\n${document.documentElement.outerHTML}\n`;
  await page.close();
  return html;
}
