# wojak-world

Windows Tauri app that indexes and searches a local wojak image library in place. The images live outside this repo under `C:\Users\William\Wojak-World`.

Working rules are in [AGENTS.md](AGENTS.md). Architecture is in [docs/architecture.md](docs/architecture.md).

## Build on Windows

This app is Windows-only. It needs Windows 10 or later, the WebView2 runtime (already present on Windows 11), Node.js 22 or newer, and Rust 1.90 or newer. Tauri 2.12's minimum Rust version is 1.90.

```powershell
npm install
npm run tauri dev
```

`npm run tauri build` writes an NSIS installer under `src-tauri\target\release\bundle\nsis\`.

The first launch asks to index the library folders. The defaults are `C:\Users\William\Wojak-World\All Wojaks` and `C:\Users\William\Wojak-World\Wojakland Archive`. The app does not scan on later launches. Press Refresh to pick up new files. SQLite and thumbnails stay in the Tauri app-local data directory, not in the library.
