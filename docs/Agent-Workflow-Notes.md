# Agent workflow notes

When an expected command fails, record the expected command, the failure, what actually works, and the date.

## 2026-11-03

`cargo check --manifest-path src-tauri/Cargo.toml` on the Linux agent VM failed before app code was typechecked:

```
The system library `gdk-3.0` required by crate `gdk-sys` was not found.
```

Tauri's crate graph asks for GTK on Linux even though this app is Windows-only. `libgtk-3-dev` and `libwebkit2gtk-4.1-dev` are what let `cargo check` compile here. `sudo apt-get install` of the usual Tauri Linux package list still exited 1: `fuse3` stopped on the `/etc/fuse.conf` conffile prompt because the install had no terminal. The GTK and WebKit packages were configured before that prompt. The owner does not need those packages. On Windows, `npm run tauri dev` is the build.

`npm run build` with Vite 8's default LightningCSS minify failed on 98.css:

```
[lightningcss minify] Invalid media query
@media (not(hover))
```

Browsers already ignore that query. `build.cssMinify: false` in `vite.config.ts` is what makes `npm run build` succeed. Do not turn LightningCSS minify back on for this stylesheet.

Default `cargo check --manifest-path src-tauri/Cargo.toml` also fails on this image before the GTK error, because rustc is 1.83.0:

```
feature `edition2024` is required
```

`rustup toolchain install 1.90.0` then `cargo +1.90.0 check --manifest-path src-tauri/Cargo.toml` typechecks. `cargo +1.90.0 fmt --manifest-path src-tauri/Cargo.toml -- --check` passes. Installing `libgtk-3-dev` and `libwebkit2gtk-4.1-dev` still exits 1 on the `fuse3` `/etc/fuse.conf` prompt. Those -dev packages were configured anyway, and the 1.90 check then succeeded.
