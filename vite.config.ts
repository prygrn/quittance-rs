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
    // Même cible que `tsconfig.json` : le code utilise des API ES2022 comme `Object.hasOwn`,
    // que la transpilation ne remplace pas, absentes des WebKit antérieurs à Safari 15.4.
    target: "es2022",
  },
});
