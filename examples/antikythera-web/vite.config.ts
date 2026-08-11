import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [
    vue(),
  ],
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  server: {
    fs: {
      allow: [
        resolve(__dirname, ".."),
        // The `file:` dependency is a junction to ../../npm/antikythera-sdk;
        // its real path must be servable in dev (module + .wasm assets).
        resolve(__dirname, "../../npm"),
      ],
    },
  },
  assetsInclude: ["**/*.wasm"],
  optimizeDeps: {
    exclude: ['antikythera-agent'],
  },
  // The jco-transpiled component module uses top-level await (ES2022); the
  // default build target (es2020) cannot emit it.
  build: {
    target: 'es2022',
  },
});
