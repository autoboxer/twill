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
            'lucide:bold',
            'lucide:book-open-check',
            'lucide:braces',
            'lucide:calendar-sync',
            'lucide:calendar-x-2',
            'lucide:check',
            'lucide:chevron-down',
            'lucide:chevron-right',
            'lucide:chevron-up',
            'lucide:circle-alert',
            'lucide:code',
            'lucide:code-xml',
            'lucide:eye',
            'lucide:folder',
            'lucide:gauge',
            'lucide:heading',
            'lucide:heading-1',
            'lucide:heading-2',
            'lucide:heading-3',
            'lucide:image-off',
            'lucide:image-plus',
            'lucide:inbox',
            'lucide:info',
            'lucide:italic',
            'lucide:keyboard',
            'lucide:layers-3',
            'lucide:layout-template',
            'lucide:library',
            'lucide:link',
            'lucide:list',
            'lucide:list-checks',
            'lucide:list-ordered',
            'lucide:loader-circle',
            'lucide:message-circle-check',
            'lucide:message-circle-question',
            'lucide:minus',
            'lucide:panels-top-left',
            'lucide:pencil',
            'lucide:pilcrow',
            'lucide:plus',
            'lucide:redo-2',
            'lucide:refresh-cw',
            'lucide:remove-formatting',
            'lucide:rotate-ccw',
            'lucide:settings',
            'lucide:settings-2',
            'lucide:shield-check',
            'lucide:sigma',
            'lucide:sparkles',
            'lucide:square-code',
            'lucide:square-pen',
            'lucide:strikethrough',
            'lucide:tag',
            'lucide:text-quote',
            'lucide:trash-2',
            'lucide:type',
            'lucide:underline',
            'lucide:undo-2',
            'lucide:x'
          ],
          scan: false,
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
