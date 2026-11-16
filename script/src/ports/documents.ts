// Port: the documents search reads, one for each feature, rule, background, scenario and examples block.

export interface SearchDocument {
  readonly kind: "feature" | "rule" | "background" | "scenario" | "scenario-outline" | "examples";
  readonly name: string;
  /** Null for a feature's own entry. */
  readonly feature: string | null;
  /** Null for a feature, a rule, and a block outside any rule. */
  readonly rule: string | null;
  /** The name of an examples block's outline; null for every other entry. */
  readonly outline: string | null;
  /** An examples block's place among its outline's, from 1; null for every other entry. */
  readonly place: number | null;
  readonly steps: readonly string[];
  readonly tags: readonly string[];
  readonly cells: readonly string[];
  readonly page: string;
  readonly anchor: string;
}

export interface Documents {
  /** Rejects when they cannot be fetched or read. */
  load(): Promise<readonly SearchDocument[]>;
}
