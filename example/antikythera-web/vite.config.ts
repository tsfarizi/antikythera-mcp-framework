import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "node:path";
import { copyFileSync } from "node:fs";

export default defineConfig({
  plugins: [
    vue(),
    {
      name: "copy-wasm",
      closeBundle() {
        const src = resolve(__dirname, "node_modules/antikythera-agent/antikythera_wasm_bindgen_bg.wasm");
        const dest = resolve(__dirname, "dist/assets/antikythera_wasm_bindgen_bg.wasm");
        copyFileSync(src, dest);
        console.log("[vite] Copied WASM binary to dist/assets/");
      },
    },
  ],
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
      "antikythera-agent/antikythera_wasm_bindgen": resolve(
        __dirname,
        "node_modules/antikythera-agent/antikythera_wasm_bindgen.js"
      ),
    },
  },
  server: {
    fs: {
      allow: [
        resolve(__dirname, ".."),
        resolve(__dirname, "../../npm"),
      ],
    },
  },
  assetsInclude: ["**/*.wasm"],
});
