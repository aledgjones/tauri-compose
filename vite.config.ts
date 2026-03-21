import { defineConfig } from "vitest/config";
import { resolve } from "path";
import react from "@vitejs/plugin-react";
import eslintPlugin from "@nabla/vite-plugin-eslint";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  // eslintPlugin spawns a file-watcher that outlives the test process.
  // Skip it during Vitest runs to prevent the "close timed out" hang.
  plugins: [react(), ...(process.env.VITEST ? [] : [eslintPlugin()])],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
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
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      input: {
        debugger: resolve(__dirname, "debugger.html"),
        index: resolve(__dirname, "index.html"),
        instrumentPicker: resolve(__dirname, "instrument-picker.html"),
        layoutSettings: resolve(__dirname, "layout-settings.html"),
        playerSettings: resolve(__dirname, "player-settings.html"),
        pluginHostPoc: resolve(__dirname, "plugin-host-poc.html"),
        pluginManager: resolve(__dirname, "plugin-manager.html"),
        pluginSettings: resolve(__dirname, "plugin-settings.html"),
        preferences: resolve(__dirname, "preferences.html"),
        projectSettings: resolve(__dirname, "project-settings.html"),
      },
    },
  },
  test: {
    globals: true,
    environment: "happy-dom",
    include: ["src/**/*.test.{ts,tsx}"],
    coverage: {
      provider: "v8" as const,
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "src/**/*.test.{ts,tsx}",
        "src/__mocks__/**",
        "src/vite-env.d.ts",
      ],
      thresholds: {
        statements: 80,
        branches: 80,
        functions: 80,
        lines: 80,
      },
    },
    setupFiles: ["./src/test-setup.ts"],
  },
}));
