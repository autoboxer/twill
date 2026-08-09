import vue from '@vitejs/plugin-vue';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;
const devServerPort = 1420;
const hotReloadPort = 1421;

export default defineConfig({
  plugins: [ vue() ],
  clearScreen: false,
  server: {
    host: host || false,
    port: devServerPort,
    strictPort: true,
    hmr: host
      ? {
        host,
        port: hotReloadPort,
        protocol: 'ws'
      }
      : undefined,
    watch: {
      ignored: [ '**/src-tauri/**' ]
    }
  }
});
