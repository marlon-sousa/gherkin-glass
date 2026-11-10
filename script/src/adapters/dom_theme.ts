// Adapter: the theme port over the page's root and its Theme control, as templates/layout.html writes them.

import type { ThemeChoice } from "../policies/theme_choice.ts";
import type { Theme } from "../ports/theme.ts";

export class DomTheme implements Theme {
  private readonly document: Document;

  constructor(document: Document) {
    this.document = document;
  }

  show(choice: ThemeChoice): void {
    for (const radio of this.radios()) {
      radio.checked = radio.value === choice;
    }
  }

  apply(theme: "light" | "dark" | null): void {
    const root = this.document.documentElement;
    if (theme === null) {
      delete root.dataset.theme;
    } else {
      root.dataset.theme = theme;
    }
  }

  onChoose(handler: (choice: ThemeChoice) => void): void {
    for (const radio of this.radios()) {
      radio.addEventListener("change", () => {
        if (radio.checked && isChoice(radio.value)) {
          handler(radio.value);
        }
      });
    }
  }

  private radios(): HTMLInputElement[] {
    return [...this.document.querySelectorAll("input[name='ghg-theme']")].filter(
      (element): element is HTMLInputElement => element instanceof HTMLInputElement,
    );
  }
}

function isChoice(value: string): value is ThemeChoice {
  return value === "system" || value === "light" || value === "dark";
}
