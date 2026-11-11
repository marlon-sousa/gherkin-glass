// Port: the documents search reads, one for each feature, rule and scenario.

export interface SearchDocument {
  readonly kind: "feature" | "rule" | "scenario" | "scenario-outline";
  readonly name: string;
  /** Null for a feature's own entry. */
  readonly feature: string | null;
  /** Null for a feature, a rule, and a scenario outside any rule. */
  readonly rule: string | null;
  readonly steps: readonly string[];
  readonly tags: readonly string[];
  readonly page: string;
  readonly anchor: string;
}

export interface Documents {
  /** Rejects when they cannot be fetched or read. */
  load(): Promise<readonly SearchDocument[]>;
}
