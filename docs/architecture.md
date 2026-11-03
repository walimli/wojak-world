# Architecture

**Status: implemented.** This is the first cut: a Windows Tauri 2 app. The sections below are the locked behavior.

Repo: https://github.com/walimli/wojak-world

Image binaries are not in git.

## Stack

- Desktop shell: Tauri 2, Windows only.
- Frontend: Preact + Vite. The UI browses, searches, favorites, and tags.
- Backend: Rust. It owns filesystem scan and index, SQLite, thumbnail generation, and search queries.

SQLite and the thumbnail cache both live in Tauri app-local data. Neither is in the library tree, and neither is in git.

## Library roots

Default roots:

- `C:\Users\William\Wojak-World\Main Wojak`
- `C:\Users\William\Wojak-World\Wojakland`

Settings may override those roots. The Settings button edits the list. Saving does not scan. Refresh indexes the saved folders.

The app indexes files in place. It never copies, moves, or owns the image binaries. The library is about 18.5k images, mostly PNG, in flat folder trees.

## Identity

The absolute path is the primary key of an image.

Favorites, tags, and an optional `display_name` are app metadata. They can change freely in the app. Changing them does not rename or rewrite the file on disk.

A real filesystem rename or move does not auto-carry that metadata. The new location is a different path, so it is a different record. Favorites, tags, and `display_name` stay on the old path.

## Schema

SQLite file: `library.sqlite` in Tauri app-local data. Migration is `PRAGMA user_version` 1 in `src-tauri/src/db.rs`.

- `images`: `path` primary key, `basename`, `folder`, `extension`, `size`, `mtime`, `favorite`, `display_name`, `missing`, `indexed_at`.
- `tags` and `image_tags`: ordinary tables. Tags are not stored only inside the image row.
- `image_tokens` and `image_categories`: derived filename tokens, filename phrases, and the parent folder name. They are search input. This cut has no separate taxonomy editor. The detail pane shows categories as read-only suggested text.
- `image_search` plus FTS5 `image_fts` over basename, derived tokens, tags, and `display_name`. Tags remain the normal tables above. FTS is the search index, not the source of truth for tags. `display_name` is indexed so a renamed image can be found.
- `thumbnails`: cache file name, pixel size, and the source `mtime` and `size` the file was built from.
- `settings`: `roots` JSON and `setup_complete`.

Thumbnail files are `{sha256 of path}.jpg` under `thumbs/` in that same app-local directory.

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

Generate a thumbnail the first time the image appears in the grid. The grid only mounts visible cells, and each cell asks for its thumb then. Cache that file in Tauri app-local data only. Never write thumbnails, sidecars, or a cache directory into Wojak-World.

The webview loads thumbs and the detail preview from the `wojak` scheme (`http://wojak.localhost/...` on Windows). The handler serves a path only when that path is already in `images`.

## Boundaries

- Do not write into the library tree. The app does not copy, move, or own the binaries, and it does not drop cache files beside them.
- Regenerate thumbnails safely inside the Tauri app-local cache. Rebuilding a thumb replaces or recreates that cache file only.
- Do not keep the index current with a routine re-scan. There is no launch auto-scan and no folder watcher in this cut. New files are indexed only when the owner explicitly refreshes or imports. A rename, move, or delete on disk does not automatically delete the row or move favorites, tags, or `display_name`.

## UI

Implemented. The window below is the built screen.

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
- Settings, for the library roots override

Hide-missing is the show-or-hide choice in Missing files. It starts checked. Refresh is the explicit owner Refresh in Scan policy. This cut's import is Refresh after the roots are saved. There is no second import command and no folder watcher.

### Status bar

- Visible count
- Total indexed
- Favorites count
- Selected display name or path

## File layout

- `src/` Preact UI. `styles.css` is only the grid, the panes, and the full-frame layout. Chrome classes come from the `98.css` package.
- `src-tauri/src/lib.rs` Tauri setup, the `wojak` scheme, and commands.
- `src-tauri/src/db.rs` schema, migration, favorites, tags, and `display_name`.
- `src-tauri/src/scan.rs` setup index and Refresh. Read-only on the library.
- `src-tauri/src/search.rs` FTS query.
- `src-tauri/src/tokens.rs` filename tokens and suggested categories.
- `src-tauri/src/thumbs.rs` JPEG thumbs in app-local data.
- `src-tauri/capabilities/default.json` window permissions for Minimize, Maximize/Restore, Close, and dragging the title bar.

The frame is one undecorated Tauri window. The HTML title bar is the window chrome.
