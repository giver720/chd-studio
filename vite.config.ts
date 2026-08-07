import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a node global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1425,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1426 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
