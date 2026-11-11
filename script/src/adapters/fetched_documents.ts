// Adapter: the documents port over the site's assets/search.json, fetched over HTTP.

import type { Documents, SearchDocument } from "../ports/documents.ts";

export class FetchedDocuments implements Documents {
  private readonly address: URL;

  constructor(address: URL) {
    this.address = address;
  }

  async load(): Promise<readonly SearchDocument[]> {
    const response = await fetch(this.address);
    if (!response.ok) {
      throw new Error(`${this.address.href}: ${response.status}`);
    }
    const found: unknown = await response.json();
    if (!Array.isArray(found)) {
      throw new Error(`${this.address.href}: not a list of documents`);
    }
    return found as SearchDocument[];
  }
}
