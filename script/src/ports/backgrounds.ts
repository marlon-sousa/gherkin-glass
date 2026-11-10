// Port: how the page shows the backgrounds its scenarios start with, and the reader's switch between the forms.

export interface Backgrounds {
  /** The form the page's HTML was written in: true for inline, false for linked. */
  writtenInline(): boolean;
  setInline(inline: boolean): void;
  /** The reader switched the option, to inline when true. */
  onSwitch(handler: (inline: boolean) => void): void;
  /** The page was shown again as it was left, such as by the back button, without its script starting again. */
  onReturn(handler: () => void): void;
}
