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
- `tags` and `image_tags`: ordinary tables. Tags are not stored only inside the image row. Tagged only matches a path that has at least one `image_tags` row. Derived filename categories stay in `image_categories` and do not satisfy that filter.
- `image_tokens` and `image_categories`: derived filename tokens, filename phrases, and the parent folder name. They are search input. This cut has no separate taxonomy editor. The detail pane shows categories as read-only suggested text.
- `image_search` plus FTS5 `image_fts` over basename, derived tokens, tags, and `display_name`. Tags remain the normal tables above. FTS is the search index, not the source of truth for tags. `display_name` is indexed so a renamed image can be found. The search box builds the `MATCH` expression. See Search.
- `thumbnails`: cache file name, pixel size, and the source `mtime` and `size` the file was built from.
- `settings`: `roots` JSON and `setup_complete`.

Thumbnail files are `{sha256 of path}.jpg` under `thumbs/` in that same app-local directory.

## Search

Submitted search text is parsed into an FTS5 `MATCH` expression. The typed text is not passed through to SQLite.

An empty submission matches every indexed image the filters allow. Text that normalizes to no words matches nothing. Any other submission is a query:

- A word is a prefix term. `doomer` matches a token that starts with `doomer`.
- Words separated by space mean AND. The word `AND` means the same thing. A single non-digit letter is still ignored, as it was before operators.
- `OR` combines alternatives. `NOT` excludes the part on its right, as in `doomer NOT npc`. A leading `NOT` is rejected. `AND`, `OR`, and `NOT` ignore case.
- A quoted phrase matches those words in order. It does not add a prefix wildcard.
- Parentheses group the expression.
- An incomplete or unbalanced query does not run. A message dialog shows a short explanation. Database and FTS error text stays off that dialog.

Favorites only, Tagged only, and Hide missing sit beside the box. They are filters, not words in the query. The box is a draft until Search or Enter.

Search reads basename, filename tokens, tags, and display name. Building the query does not change filename token indexing.

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
- Search button
- Search help (`?`), listing AND, OR, NOT, phrases, and groups
- Favorites-only toggle
- Tagged-only toggle
- Hide-missing toggle
- Refresh
- Settings, for the library roots override

The search box holds a draft. Search runs when the user clicks Search or presses Enter in the box. Typing does not search by itself.

Favorites only, Tagged only, and Hide missing each run that search again with the submitted text. The ready screen runs one search on load: empty text, Hide missing checked, Favorites only and Tagged only unchecked. Refresh indexes, then runs the same search with the submitted text and the current checkboxes. Busy disables the toolbar controls.

Tagged only keeps rows with at least one manual tag in `image_tags`. Filename tokens and suggested categories do not count. Clearing the last manual tag on the selected image while Tagged only is on removes that image from the grid and clears the selection.

Submitted text searches basename, filename tokens, manual tags, and `display_name`. The query language is in Search above.

Hide-missing is the show-or-hide choice in Missing files. It starts checked. Refresh is the explicit owner Refresh in Scan policy. This cut's import is Refresh after the roots are saved. There is no second import command and no folder watcher.

### Status bar

- Visible count
- Total indexed
- Favorites count
- Selected display name or path

## File layout

- `src/` Preact UI. `styles.css` is only the grid, the panes, and the full-frame layout. Chrome classes come from the `98.css` package.
- `src-tauri/src/lib.rs` Tauri setup, the `wojak` scheme, and commands.
- `src-tauri/src/db.rs` schema, migration, favorites, tags, `display_name`, and filtered search.
- `src-tauri/src/scan.rs` setup index and Refresh. Read-only on the library.
- `src-tauri/src/search.rs` runs the search-box query, then the favorites, tagged, and missing filters.
- `src-tauri/src/tokens.rs` filename tokens, suggested categories, and the search-box builder.
- `src-tauri/src/thumbs.rs` JPEG thumbs in app-local data.
- `src-tauri/capabilities/default.json` window permissions for Minimize, Maximize/Restore, Close, and dragging the title bar.

The frame is one undecorated Tauri window. The HTML title bar is the window chrome.
