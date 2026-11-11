// Service: the reader's choice of theme, shown on the control, applied to the page and remembered.

import { THEME_KEY, keptChoice, themeAttribute } from "../policies/theme_choice.ts";
import type { Memory } from "../ports/memory.ts";
import type { Theme } from "../ports/theme.ts";

export function startThemeSwitching(theme: Theme, memory: Memory): void {
  const showKept = (): void => {
    const choice = keptChoice(memory.get(THEME_KEY));
    theme.show(choice);
    theme.apply(themeAttribute(choice));
  };
  showKept();
  // A choice made on another page since this one was cached.
  theme.onRestore(showKept);
  theme.onChoose((chosen) => {
    theme.apply(themeAttribute(chosen));
    memory.set(THEME_KEY, chosen);
  });
}
