import { defineConfig } from "vite";
import preact from "@preact/preset-vite";
// @ts-expect-error type error without @types/node package
import process from "node:process";

const host = process.env.TAURI_DEV_HOST;

// Vite options tailored for Tauri. The dev server port is fixed because
// Tauri loads this URL, and src-tauri is ignored so Rust rebuilds do not
// restart the frontend.
export default defineConfig(() => ({
  plugins: [preact()],
  clearScreen: false,
  // 98.css ships `@media (not(hover))`. Vite 8 minifies CSS with LightningCSS,
  // which rejects that query. Browsers already ignore the rule. Skip CSS minify
  // so `npm run build` keeps the stylesheet intact.
  build: {
    cssMinify: false,
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
