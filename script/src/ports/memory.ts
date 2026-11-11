// Port: what the browser keeps for the reader between visits.

export interface Memory {
  /** Null when nothing is kept, or nothing can be. */
  get(key: string): string | null;
  /** Keeps nothing when the browser does not allow it. */
  set(key: string, value: string): void;
}
