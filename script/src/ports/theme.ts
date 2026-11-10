// Port: the page's theme, and the control the reader chooses it with.

import type { ThemeChoice } from "../policies/theme_choice.ts";

export interface Theme {
  /** Shows `choice` as the control's chosen one. */
  show(choice: ThemeChoice): void;
  /** Draws the page in `theme`, or in the system's when it is null. */
  apply(theme: "light" | "dark" | null): void;
  onChoose(handler: (choice: ThemeChoice) => void): void;
}
