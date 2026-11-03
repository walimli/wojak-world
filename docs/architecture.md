# Architecture

**Status: intended. Not yet implemented.** This repo has no application code yet. The sections below are the locked target, not a description of a running app.

Repo: https://github.com/walimli/wojak-world

Image binaries are not in git.

## Library on disk

The library stays on disk under `C:\Users\William\Wojak-World` (All Wojaks + Wojakland Archive). It is about 18.5k images, mostly PNG, in flat folder trees.

The app indexes paths only. It never copies, moves, or owns the image binaries.

## Stack

- Desktop shell: Tauri 2, Windows only.
- Frontend: Preact + Vite. The UI is for browse, search, favorite, and tag.
- Backend: Rust. It owns filesystem scan and index, SQLite, thumbnail generation, and search queries.

## Metadata and thumbnail cache

SQLite lives in Tauri app-local data. It stores:

- paths
- favorites
- tags
- derived filename tokens and categories
- thumbnail metadata

The thumbnail image cache also lives in app-local data only. Never write thumbnails into Wojak-World.

## Domain

Image records are keyed by path. Related data on those records:

- favorites
- tags
- derived tokens (and suggested categories) taken from descriptive filenames

Manual tags refine search and suggested categories. The first mature cut has no separate taxonomy editor.

## Boundaries

- Do not write into the library tree.
- Regenerate thumbnails safely, in the app-local cache only.
- A re-scan detects adds, moves, and deletes.
