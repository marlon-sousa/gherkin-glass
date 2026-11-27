// The settings that name src/adapters/jsx/jsx_runtime.ts, which esbuild's bundle and tsc must both give.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

function read<Read>(name: string): Read {
  return JSON.parse(readFileSync(new URL(`../../../${name}`, import.meta.url), "utf8")) as Read;
}

test("the bundle compiles JSX with the runtime tsconfig.json names, which tsc and tsx read", () => {
  const { jsx, jsxImportSource } = read<{ compilerOptions: { jsx: string; jsxImportSource: string } }>("tsconfig.json").compilerOptions;
  assert.deepEqual({ jsx, jsxImportSource }, { jsx: "react-jsx", jsxImportSource: "#jsx" });
  const { bundle } = read<{ scripts: { bundle: string } }>("package.json").scripts;
  assert.ok(bundle.includes(" --jsx=automatic --jsx-import-source=#jsx "), bundle);
});

test("the runtime is reached at the name tsconfig.json gives", () => {
  const { imports } = read<{ imports: Record<string, string> }>("package.json");
  assert.equal(imports["#jsx/jsx-runtime"], "./src/adapters/jsx/jsx_runtime.ts");
});
