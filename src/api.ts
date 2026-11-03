import { invoke } from "@tauri-apps/api/core";
import type {
  ImageDetail,
  LibraryState,
  ScanReport,
  SearchResponse,
} from "./types";

export function libraryState(): Promise<LibraryState> {
  return invoke("library_state");
}

export function setRoots(roots: string[]): Promise<string[]> {
  return invoke("set_roots", { roots });
}

export function runSetup(roots: string[]): Promise<ScanReport> {
  return invoke("run_setup", { roots });
}

export function refresh(): Promise<ScanReport> {
  return invoke("refresh");
}

export function search(
  query: string,
  favoritesOnly: boolean,
  taggedOnly: boolean,
  hideMissing: boolean,
): Promise<SearchResponse> {
  return invoke("search", { query, favoritesOnly, taggedOnly, hideMissing });
}

export function imageDetail(path: string): Promise<ImageDetail> {
  return invoke("image_detail", { path });
}

export function setFavorite(path: string, favorite: boolean): Promise<boolean> {
  return invoke("set_favorite", { path, favorite });
}

export function setDisplayName(
  path: string,
  displayName: string | null,
): Promise<string | null> {
  return invoke("set_display_name", { path, displayName });
}

export function setTags(path: string, tags: string[]): Promise<string[]> {
  return invoke("set_tags", { path, tags });
}

export function errorText(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message.trim()) return error.message;
  return "Something went wrong.";
}
