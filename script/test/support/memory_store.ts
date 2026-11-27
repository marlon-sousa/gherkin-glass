// A fake of the store port, holding the site's data in memory, for the tests of what draws from it.

import type { Descriptor, PageIndex, Store, StoreObject, VersionList } from "../../src/ports/store.ts";

export class MemoryStore implements Store {
  readonly objects: ReadonlyMap<string, StoreObject>;
  /** The objects asked for, in order. */
  readonly asked: string[] = [];

  constructor(
    readonly held: Descriptor,
    objects: Readonly<Record<string, StoreObject>>,
    readonly list: VersionList | null = null,
    readonly index: PageIndex | null = null,
  ) {
    this.objects = new Map(Object.entries(objects));
  }

  async descriptor(): Promise<Descriptor> {
    return this.held;
  }

  async object(name: string): Promise<StoreObject> {
    this.asked.push(name);
    const found = this.objects.get(name);
    if (found === undefined) {
      throw new Error(`no object ${name}`);
    }
    return found;
  }

  async versions(): Promise<VersionList> {
    if (this.list === null) {
      throw new Error("no versions.json");
    }
    return this.list;
  }

  async pages(): Promise<PageIndex> {
    if (this.index === null) {
      throw new Error("no pages.json");
    }
    return this.index;
  }
}
