// Port: the checkbox that chooses whether the backgrounds inside the scenarios or those in their own place are expanded, and the reader's switch of it.

export interface Backgrounds {
  /** Whether the page's HTML was written with the box checked, which expands the backgrounds inside the scenarios. */
  writtenInline(): boolean;
  setInline(inline: boolean): void;
  /** The reader checked the box when true, or unchecked it. */
  onSwitch(handler: (inline: boolean) => void): void;
  /** The page was shown again as it was left, such as by the back button, without its script starting again. */
  onReturn(handler: () => void): void;
}
