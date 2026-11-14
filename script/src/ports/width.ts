// Port: whether the window is narrow, and when that changes.

export interface WindowWidth {
  narrow(): boolean;
  /** The window was resized across the narrow width, either way. */
  onChange(handler: (narrow: boolean) => void): void;
}
