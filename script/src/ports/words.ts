// Port: gherkin-glass's own words, each message of the catalogue by its id, with feature text put in its places.

/** A run of words, of the catalogue or of feature text, which is isolated from the words around it. */
export interface WordPart {
  readonly text: string;
  readonly isolated: boolean;
}

export type Parts = readonly WordPart[];

export type Argument = string | number;

export interface Words {
  /** The message `id`, with `args` in its places; a message the catalogue does not hold reads its id. */
  text(id: string, args?: Readonly<Record<string, Argument>>): string;
  /** The message `id`, with `args` in its places and, in the places `texts` names, their parts. */
  parts(id: string, args: Readonly<Record<string, Argument>>, texts: Readonly<Record<string, Parts>>): Parts;
}
