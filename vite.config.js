import { defineConfig } from "vite";
import { fileURLToPath } from "node:url";

const platform = process.env.TAURI_ENV_PLATFORM || (process.platform === "win32" ? "windows" : process.platform);
const windowsThemes = platform === "windows";

export default defineConfig({
  define: { __LF_WINDOWS_THEMES__: windowsThemes },
  plugins: [{
    name: "windows-appearance-styles",
    transformIndexHtml: {
      order: "pre",
      handler(html) {
        return windowsThemes ? html : html.replace(/<link\b[^>]*data-windows-appearance[^>]*>\s*/g, "");
      },
    },
  }],
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
        remote: fileURLToPath(new URL("./src/remote.html", import.meta.url)),
        ...(process.env.TAURI_ENV_PLATFORM !== "android" && {
          library: fileURLToPath(new URL("./src/library.html", import.meta.url)),
        }),
      },
    },
  },
});
