import { defineConfig } from "vite";
import { fileURLToPath } from "node:url";

export default defineConfig({
  root: "src",
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL("./src/index.html", import.meta.url)),
        library: fileURLToPath(new URL("./src/library.html", import.meta.url)),
      },
    },
  },
});
