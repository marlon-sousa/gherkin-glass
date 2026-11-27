// What the browser checks share: Playwright and axe-core from script/'s Node
// packages, the rules axe-core checks, the ghg binary, ghg serve, sites built
// and served, and the pages of a site and how to open one.

import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, relative, resolve, sep } from "node:path";
import { createInterface } from "node:readline";

import { pageByName, targetAddress } from "../../script/src/policies/address.ts";

// The Node packages are script/'s, installed from its lockfile.
const require = createRequire(resolve("script/package.json"));

export const { chromium } = require("playwright");
export const axeSource = require.resolve("axe-core/axe.min.js");

// The rules the check fails on: WCAG 2.2 A and AA, and the enhanced contrast
// of AAA, leaving out the rules axe-core tags experimental or deprecated.
const tags = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"];
const extraRules = ["color-contrast-enhanced"];

// Runs axe-core on the page and returns its violations, each as one line per
// element. axe-core must already be loaded in the page, as the checks'
// contexts load it.
export async function violations(page) {
  const found = await page.evaluate(
    async ({ tags, extraRules }) => {
      const rules = window.axe
        .getRules(tags)
        .filter((rule) => !rule.tags.includes("experimental") && !rule.tags.includes("deprecated"))
        .map((rule) => rule.ruleId)
        .concat(extraRules);
      const results = await window.axe.run(document, {
        runOnly: { type: "rule", values: rules },
        resultTypes: ["violations"],
      });
      return results.violations.flatMap((violation) =>
        violation.nodes.map((node) => `${violation.id}: ${node.target.join(" ")}: ${node.failureSummary}`),
      );
    },
    { tags, extraRules },
  );
  return found;
}

// GHG names another ghg binary than target/debug/ghg.
export const ghg = resolve(process.env.GHG ?? "target/debug/ghg");

// Runs ghg serve with `args` in `cwd`, and resolves once it listens, with the
// address it prints. A run that stops before it listens rejects, with what it
// printed to standard error.
export function serve(args, cwd) {
  const server = spawn(ghg, ["serve", ...args], { cwd, stdio: ["ignore", "pipe", "pipe"] });
  let errors = "";
  server.stderr.on("data", (chunk) => {
    errors += chunk;
  });
  return new Promise((done, fail) => {
    const lines = createInterface({ input: server.stdout });
    lines.once("line", (line) => {
      const address = /^The site is served at (\S+)\. /.exec(line)?.[1];
      if (address) {
        done({ server, address });
      } else {
        server.kill();
        fail(new Error(`ghg serve printed ${JSON.stringify(line)}`));
      }
    });
    server.once("exit", (code) => fail(new Error(`ghg serve stopped with status ${code}: ${errors}`)));
  });
}

// Runs each of `commands`, a fixture's folder below tests/fixtures and the
// arguments of ghg, such as ["build"], into one new temporary folder, serves
// it, and resolves with its address and a way to stop serving and remove it.
export async function serveBuilt(commands) {
  const output = mkdtempSync(join(tmpdir(), "ghg-built-"));
  for (const { folder, args } of commands) {
    const run = spawnSync(ghg, [...args, "--output", output], { cwd: resolve("tests/fixtures", folder), encoding: "utf8" });
    if (run.status !== 0) throw new Error(`ghg ${args.join(" ")} in tests/fixtures/${folder}: ${run.stderr}`);
  }
  const { server, address } = await serve(["--no-build", "--output", output, "--port", "0"], output);
  return {
    address,
    stop: async () => {
      await stop(server);
      rmSync(output, { recursive: true, force: true });
    },
  };
}

// Stops a server `serve` started.
export async function stop(server) {
  if (server.exitCode === null && server.signalCode === null) {
    const exited = new Promise((done) => server.once("exit", done));
    server.kill();
    await exited;
  }
}

// Every page of the site in the output folder `folder`, by the path of its
// HTML from the folder, with the address of the page after it for a page a
// shell draws, such as `v1.0.0/index.html#billing/refunds`, in order: each
// page of HTML ghg wrote, and each page a shell draws from its descriptor,
// its front page, its folders, its features, its tags, its step sentences and
// its search results. A shell is a page whose `html` names its descriptor.
export function sitePages(folder) {
  const pages = [];
  for (const path of htmlFiles(folder)) {
    const html = readFileSync(join(folder, path), "utf8");
    const named = /<html[^>]* data-ghg-descriptor="([^"]+)"/.exec(html)?.[1];
    if (named === undefined) {
      pages.push(path);
      continue;
    }
    const descriptor = JSON.parse(readFileSync(join(folder, "data", named), "utf8"));
    pages.push(...shellPageNames(descriptor).map((name) => `${path}${targetAddress(pageByName(name))}`));
  }
  return pages;
}

// The names of the pages a shell of `descriptor` draws, as the store names them.
function shellPageNames(descriptor) {
  const features = descriptor.features.map((feature) => feature.page);
  const folders = new Set([""]);
  for (const page of features) {
    const parts = page.split("/").slice(0, -1);
    parts.forEach((_, index) => folders.add(`${parts.slice(0, index + 1).join("/")}/`));
  }
  const tags = descriptor.tags.map((tag) => tag.page);
  return [...[...folders].sort(), ...features, "tags/", ...tags, "steps", "search"];
}

// The files of HTML below `folder`, with `/` between folders, in order.
function htmlFiles(folder) {
  return readdirSync(folder, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".html"))
    .map((entry) => relative(folder, join(entry.parentPath, entry.name)).split(sep).join("/"))
    .sort();
}

// The address of `page`, as sitePages gives it, below the site's address,
// the parts of its path percent-encoded.
export function pageAddress(site, page) {
  const split = page.indexOf("#");
  const path = split < 0 ? page : page.slice(0, split);
  const hash = split < 0 ? "" : page.slice(split);
  return `${new URL(path.split("/").map(encodeURIComponent).join("/"), site).href}${hash}`;
}

// Opens `address` in `page`, a new document each time, and waits until its
// shell, if it is one, has drawn the page the address names.
export async function openPage(page, address) {
  await page.goto("about:blank");
  await page.goto(address);
  try {
    await page.waitForFunction(() => !document.getElementById("ghg_main")?.hasAttribute("aria-busy"));
  } catch (error) {
    throw new Error(`${address} was never drawn: ${error.message.split("\n")[0]}`);
  }
}
