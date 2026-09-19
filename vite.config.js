import vue from '@vitejs/plugin-vue';
import ui from '@nuxt/ui/vite';
import { defineConfig } from 'vite';

import { uiTheme } from './src/config/ui';

const host = process.env.TAURI_DEV_HOST;
const devServerPort = 1420;
const hotReloadPort = 1421;

export default defineConfig({
  plugins: [
    vue(),
    ui({
      colorMode: false,
      dts: false,
      icon: {
        clientBundle: {
          scan: {
            // Include full icon names in JavaScript registries as well as templates
            globInclude: [ 'src/**/*.{vue,js}' ]
          },
          sizeLimitKb: 256
        }
      },
      ui: uiTheme
    })
  ],
  clearScreen: false,
  optimizeDeps: {
    exclude: [
      '@tiptap/core',
      '@tiptap/pm'
    ]
  },
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
