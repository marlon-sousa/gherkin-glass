// Port: the site's data, its descriptors, objects, list of versions and index of pages.

export type BlockKind = "feature" | "background" | "rule" | "scenario" | "scenario-outline" | "examples";
export type StepKind = "given" | "when" | "then";
export type PartKind = "text" | "parameter" | "number" | "placeholder";

/** A run of a step's text as it is highlighted; only a parameter holds parts. */
export interface Part {
  readonly kind: PartKind;
  readonly text: string;
  readonly parts: readonly Part[];
}

/** A table's rows, its header first, each a list of its cells. */
export type Table = readonly (readonly string[])[];

export interface DocString {
  readonly media_type: string | null;
  readonly content: string;
}

export interface Step {
  readonly keyword: string;
  readonly text: string;
  readonly kind: StepKind;
  readonly sentence: string;
  readonly slots: readonly string[];
  readonly parts: readonly Part[];
  readonly table: Table | null;
  readonly doc_string: DocString | null;
}

/** A scenario an examples block's row produces. */
export interface Run {
  readonly anchor: string;
  readonly steps: readonly Step[];
}

export interface Block {
  readonly kind: BlockKind;
  readonly keyword: string;
  /** Whether the block's content starts with the note of the keyword its file writes. */
  readonly note: boolean;
  readonly name: string;
  readonly description: string;
  readonly tags: readonly string[];
  /** Empty text for the feature, which is its page. */
  readonly anchor: string;
  readonly hash: string;
  readonly blocks: readonly Block[];
  readonly steps: readonly Step[];
  readonly examples: readonly Block[];
  /** Null for every block but an examples block. */
  readonly place: number | null;
  readonly table: Table | null;
  readonly runs: readonly Run[];
}

/** A feature file's content. */
export interface StoreObject {
  readonly format: number;
  readonly language: string;
  readonly lang: string;
  readonly feature: Block;
}

export interface FeatureEntry {
  readonly path: string;
  readonly page: string;
  readonly object: string;
  readonly lines: readonly number[];
  readonly source: string | null;
}

export interface TagLink {
  readonly address: string;
  readonly text: string;
}

export interface TagEntry {
  readonly tag: string;
  readonly page: string;
  readonly link: TagLink | null;
}

export interface StartStates {
  readonly collapse_rules: boolean;
  readonly collapse_scenarios: boolean;
  readonly show_tree: boolean;
  readonly inline_backgrounds: boolean;
}

export interface Sentences {
  readonly given: readonly string[];
  readonly when: readonly string[];
  readonly then: readonly string[];
}

export type VersionKind = "development" | "release" | "pre-release";

/** A site's, a version's or a preview's; the members the script never reads are left out. */
export interface Descriptor {
  readonly format: number;
  readonly ghg: string;
  readonly version: string | null;
  /** What its name makes the version; null for a site without versions and a preview. */
  readonly version_kind: VersionKind | null;
  readonly preview: string | null;
  readonly title: string;
  readonly only: boolean;
  readonly start: StartStates;
  readonly features: readonly FeatureEntry[];
  readonly tags: readonly TagEntry[];
  readonly sentences: Sentences;
}

/** The members of `data/versions.json` the script reads. */
export interface VersionList {
  readonly format: number;
  readonly development: readonly string[];
  readonly releases: readonly string[];
  readonly pre_releases: readonly string[];
  /** Each holds the names of one precedence, newest first. */
  readonly precedence: readonly (readonly string[])[];
}

export interface PageObject {
  readonly object: string;
  readonly versions: readonly string[];
}

export interface PageEntry {
  readonly page: string;
  readonly objects: readonly PageObject[];
}

export interface PageIndex {
  readonly format: number;
  readonly pages: readonly PageEntry[];
}

/** Each rejects when its file cannot be fetched, or is not what the script reads, and is fetched again when asked again. */
export interface Store {
  /** The descriptor of the shell the script draws. */
  descriptor(): Promise<Descriptor>;
  /** The object named `name`, without `.json`. */
  object(name: string): Promise<StoreObject>;
  versions(): Promise<VersionList>;
  pages(): Promise<PageIndex>;
}
