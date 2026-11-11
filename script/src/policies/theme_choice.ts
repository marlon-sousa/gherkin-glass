// Policy: the theme a reader can choose, how the choice is kept, and the theme it shows.

/** The theme choice is kept under this key for every page of the origin. templates/layout.html reads it too. */
export const THEME_KEY = "gherkin-glass:theme";

export type ThemeChoice = "system" | "light" | "dark";

/** What was kept, or `system` when nothing or anything else was. */
export function keptChoice(kept: string | null): ThemeChoice {
  return kept === "light" || kept === "dark" ? kept : "system";
}

/** The page's `data-theme`, or null for the system's choice, which the stylesheet follows itself. */
export function themeAttribute(choice: ThemeChoice): "light" | "dark" | null {
  return choice === "system" ? null : choice;
}
