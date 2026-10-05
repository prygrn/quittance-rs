import { defineConfig } from "vite";

// Configuration recommandée par Tauri pour un front Vite (https://v2.tauri.app/start/frontend/vite/).
export default defineConfig({
  // Garde visibles les erreurs de compilation Rust affichées par `tauri dev`.
  clearScreen: false,
  server: {
    // Doit correspondre à `build.devUrl` de `src-tauri/tauri.conf.json`.
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    // Tauri affiche le front dans WebKit (Linux, macOS) et WebView2, basé sur Chromium (Windows).
    target: ["safari13", "chrome105"],
  },
});
