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
            'lucide:archive',
            'lucide:archive-restore',
            'lucide:arrow-left',
            'lucide:book-open-check',
            'lucide:check',
            'lucide:chevron-right',
            'lucide:circle-alert',
            'lucide:folder',
            'lucide:inbox',
            'lucide:layers-3',
            'lucide:library',
            'lucide:loader-circle',
            'lucide:pencil',
            'lucide:plus',
            'lucide:refresh-cw',
            'lucide:settings',
            'lucide:settings-2',
            'lucide:square-pen',
            'lucide:tag',
            'lucide:trash-2',
            'lucide:x'
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
