import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import { analyzer } from "vite-bundle-analyzer";

const plugins = [];

if (process.env.ANALYZE_BUNDLE) {
  plugins.push(analyzer({ openAnalyzer: false }));
}

plugins.push(react(), wasm(), topLevelAwait());

// https://vite.dev/config/
export default defineConfig({
  base: "/ios-xr-switch-config-builder/",
  plugins,
  build: {
    // Keep assets at dist/ instead of dist/assets
    assetsDir: "",
  },
});
