# Architecture

**Status: intended. Not yet implemented.** This repo has no application code yet. The sections below are the locked target, not a description of a running app.

Repo: https://github.com/walimli/wojak-world

Image binaries are not in git.

## Stack

- Desktop shell: Tauri 2, Windows only.
- Frontend: Preact + Vite. The UI browses, searches, favorites, and tags.
- Backend: Rust. It owns filesystem scan and index, SQLite, thumbnail generation, and search queries.

SQLite and the thumbnail cache both live in Tauri app-local data. Neither is in the library tree, and neither is in git.

## Library roots

Default roots:

- `C:\Users\William\Wojak-World\All Wojaks`
- `C:\Users\William\Wojak-World\Wojakland Archive`

Settings may override those roots.

The app indexes files in place. It never copies, moves, or owns the image binaries. The library is about 18.5k images, mostly PNG, in flat folder trees.

## Identity

The absolute path is the primary key of an image.

Favorites, tags, and an optional `display_name` are app metadata. They can change freely in the app. Changing them does not rename or rewrite the file on disk.

A real filesystem rename or move does not auto-carry that metadata. The new location is a different path, so it is a different record. Favorites, tags, and `display_name` stay on the old path.

## Schema sketch

High level and intended only. This is not a migration and not a final column list.

- `images`: `path` primary key, `basename`, `folder`, `extension`, `size`, `mtime`, `favorite`, `display_name`, a missing flag, and the other index fields this cut needs.
- `tags` and `image_tags`: ordinary tables. Tags are not stored only inside the image row.
- SQLite FTS5 over `basename` and tags. Derived filename tokens feed FTS. Tags remain the normal tables above. FTS is the search index, not the source of truth for tags.
- Thumbnail metadata points at cache files in Tauri app-local data.

Derived filename tokens are search input. This cut has no separate taxonomy editor.

## Scan policy

The library is static.

- Build the initial index at setup.
- After setup, new files enter the index only through an explicit Refresh or import, and only when the owner runs it.
- This mature cut does not scan on launch and does not watch folders.

Refresh and import are owner actions. They are not a routine background re-scan, and the app does not treat continuous detection of adds, moves, and deletes as the normal mode.

## Missing files

When a file is no longer at its indexed path, keep the row. Keep its favorites and tags. Mark the row missing. The UI may hide missing rows or show them.

A filesystem rename or move is not applied onto the old row. The old path can remain a missing record, with its metadata still attached to that path.

## Thumbnails

Generate a thumbnail the first time the image appears in the grid. Cache that file in Tauri app-local data only. Never write thumbnails, sidecars, or a cache directory into Wojak-World.

## Boundaries

- Do not write into the library tree. The app does not copy, move, or own the binaries, and it does not drop cache files beside them.
- Regenerate thumbnails safely inside the Tauri app-local cache. Rebuilding a thumb replaces or recreates that cache file only.
- Do not keep the index current with a routine re-scan. There is no launch auto-scan and no folder watcher in this cut. New files are indexed only when the owner explicitly refreshes or imports. A rename, move, or delete on disk does not automatically delete the row or move favorites, tags, or `display_name`.

## UI

Intended. Not implemented. The window below is part of the locked target, not a built screen.

### Visual system

Use 98.css from npm: https://jdan.github.io/98.css/

Win98 chrome is semantic HTML and 98.css classes. Custom CSS is only for grid and layout gaps that 98.css does not cover.

### Chrome

One primary Win98 window fills the Tauri frame. It has a title bar, a toolbar, a body, and a status bar.

Title-bar Minimize, Maximize/Restore, and Close call the real Tauri window APIs.

### Body

The body layout is a search and filter toolbar at the top, then a scrollable thumbnail grid. Selecting a thumbnail opens a detail pane on the right. The pane shows a preview, the name, favorite, and tags.

### Toolbar

- Search box
- Favorites-only toggle
- Hide-missing toggle
- Refresh

Hide-missing is the show-or-hide choice in Missing files. Refresh is the explicit owner Refresh in Scan policy.

### Status bar

- Visible count
- Total indexed
- Favorites count
- Selected display name or path
