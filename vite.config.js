import vue from '@vitejs/plugin-vue';
import ui from '@nuxt/ui/vite';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;
const devServerPort = 1420;
const hotReloadPort = 1421;

export default defineConfig({
  plugins: [
    vue(),
    ui({
      colorMode: true,
      dts: false,
      icon: {
        clientBundle: {
          icons: [
            'lucide:book-open-check',
            'lucide:circle-alert',
            'lucide:inbox',
            'lucide:library',
            'lucide:loader-circle',
            'lucide:settings',
            'lucide:square-pen'
          ],
          scan: true,
          sizeLimitKb: 256
        }
      },
      ui: {
        colors: {
          neutral: 'stone',
          primary: 'moss'
        }
      }
    })
  ],
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
