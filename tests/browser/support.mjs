// What the browser checks share: Playwright and axe-core from script/'s Node
// packages, the ghg binary, and ghg serve.

import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { createInterface } from "node:readline";

// The Node packages are script/'s, installed from its lockfile.
const require = createRequire(resolve("script/package.json"));

export const { chromium } = require("playwright");
export const axeSource = require.resolve("axe-core/axe.min.js");

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
