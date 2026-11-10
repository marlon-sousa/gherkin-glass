// The service of src/services/theme_switching.ts, with fake ports: docs/BEHAVIOUR.md, rules 61 and 62.

import assert from "node:assert/strict";
import { test } from "node:test";

import { THEME_KEY, type ThemeChoice } from "../../src/policies/theme_choice.ts";
import type { Memory } from "../../src/ports/memory.ts";
import type { Theme } from "../../src/ports/theme.ts";
import { startThemeSwitching } from "../../src/services/theme_switching.ts";

class FakeTheme implements Theme {
  shown: ThemeChoice | undefined;
  applied: "light" | "dark" | null | undefined;
  choose: (choice: ThemeChoice) => void = () => {};

  show(choice: ThemeChoice): void {
    this.shown = choice;
  }

  apply(theme: "light" | "dark" | null): void {
    this.applied = theme;
  }

  onChoose(handler: (choice: ThemeChoice) => void): void {
    this.choose = handler;
  }
}

class FakeMemory implements Memory {
  readonly kept = new Map<string, string>();
  readonly works: boolean;

  constructor(works = true) {
    this.works = works;
  }

  get(key: string): string | null {
    return this.kept.get(key) ?? null;
  }

  set(key: string, value: string): void {
    if (this.works) {
      this.kept.set(key, value);
    }
  }
}

test("without a kept choice the page follows the system", () => {
  const theme = new FakeTheme();
  startThemeSwitching(theme, new FakeMemory());
  assert.equal(theme.shown, "system");
  assert.equal(theme.applied, null);
});

test("a kept choice is shown and applied as the page opens", () => {
  const memory = new FakeMemory();
  memory.kept.set(THEME_KEY, "dark");
  const theme = new FakeTheme();
  startThemeSwitching(theme, memory);
  assert.equal(theme.shown, "dark");
  assert.equal(theme.applied, "dark");
});

test("a choice is applied and kept, and choosing system again forgets the theme", () => {
  const memory = new FakeMemory();
  const theme = new FakeTheme();
  startThemeSwitching(theme, memory);
  theme.choose("light");
  assert.equal(theme.applied, "light");
  assert.equal(memory.kept.get(THEME_KEY), "light");
  theme.choose("system");
  assert.equal(theme.applied, null);
  assert.equal(memory.kept.get(THEME_KEY), "system");
});

test("when nothing can be kept the choice still holds for the page", () => {
  const theme = new FakeTheme();
  startThemeSwitching(theme, new FakeMemory(false));
  theme.choose("dark");
  assert.equal(theme.applied, "dark");
});
