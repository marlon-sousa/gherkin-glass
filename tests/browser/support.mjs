// What the browser checks share: Playwright and axe-core from script/'s Node
// packages, the rules axe-core checks, the ghg binary, and ghg serve.

import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { createInterface } from "node:readline";

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

// Stops a server `serve` started.
export async function stop(server) {
  if (server.exitCode === null && server.signalCode === null) {
    const exited = new Promise((done) => server.once("exit", done));
    server.kill();
    await exited;
  }
}
