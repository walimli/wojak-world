export interface LibraryState {
  setupComplete: boolean;
  roots: string[];
  defaultRoots: string[];
}

export interface ScanReport {
  seen: number;
  added: number;
  updated: number;
  markedMissing: number;
  errors: string[];
}

export interface ImageSummary {
  path: string;
  basename: string;
  folder: string;
  extension: string;
  size: number;
  mtime: number;
  favorite: boolean;
  displayName: string | null;
  missing: boolean;
}

export interface ImageDetail extends ImageSummary {
  tags: string[];
  categories: string[];
}

export interface SearchResponse {
  items: ImageSummary[];
  visible: number;
  total: number;
  favorites: number;
}
