// Port: waiting for the reader to pause.

export interface Timer {
  /** Calls `handler` after `milliseconds`, cancelling the call asked for before, if it has not happened. */
  after(milliseconds: number, handler: () => void): void;
  cancel(): void;
}
