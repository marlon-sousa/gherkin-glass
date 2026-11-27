// Tool: writes the licences of the packages the compiled script holds, the script's dependencies, each with its name, version, licence and full text, from its package or from `licenses/` when its package ships none.

import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const script = join(import.meta.dirname, "..");

interface Manifest {
  readonly name: string;
  readonly version: string;
  readonly license: string;
  readonly dependencies?: Readonly<Record<string, string>>;
}

function manifest(folder: string): Manifest {
  return JSON.parse(readFileSync(join(folder, "package.json"), "utf8")) as Manifest;
}

// The text of the licence `name` ships, or the one `licenses/` keeps for it, named after it with `/` as `-`.
function licenceText(name: string, folder: string): string {
  const shipped = readdirSync(folder).find((file) => /^(LICEN[CS]E|COPYING)/i.test(file));
  const file = shipped === undefined
    ? join(script, "licenses", `${name.replace("/", "-")}.txt`)
    : join(folder, shipped);
  return readFileSync(file, "utf8").replace(/^\n+/, "").trimEnd();
}

function section(name: string): string {
  const folder = join(script, "node_modules", name);
  const bundled = manifest(folder);
  if (Object.keys(bundled.dependencies ?? {}).length > 0) {
    throw new Error(`${name} has dependencies of its own, whose licences this tool does not write.`);
  }
  const heading = `${bundled.name} ${bundled.version}, under the ${bundled.license} licence`;
  return `${heading}\n\n${licenceText(name, folder)}\n`;
}

const dependencies = Object.keys(manifest(script).dependencies ?? {}).sort();
const preamble =
  "The licences of the packages bundled into gherkin-glass.js, the script of the pages gherkin-glass writes.\n";
const output = process.argv[2];
if (output === undefined) {
  throw new Error("Give the file to write.");
}
writeFileSync(output, [preamble, ...dependencies.map(section)].join("\n"));
