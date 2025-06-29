import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";
import viteShikiPlugin from "./vite-plugin-shiki.js";
import { fileURLToPath } from 'url'
import VitePluginVueDevTools from 'vite-plugin-vue-devtools'

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue({
      script: {
        defineModel: true,
        propsDestructure: true
      }
    }),
    viteShikiPlugin({
      theme: 'dark',
      lineNumbers: true,
      cache: true
    }),
    VitePluginVueDevTools()
  ],

  // Path aliases
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    open: true,
    cors: true,

    // Complete proxy configuration to solve CORS issues - unified through remote Gateway
    proxy: {
      // API proxy to remote Gateway (unified entry point)
      '/api': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        // No rewrite needed, keep /api prefix
      },
      // Health check proxy
      '/health': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
      },
      // File service proxy to remote Gateway  
      '/files': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        // No rewrite needed, keep /files prefix
      },
      // SSE events proxy to remote Gateway
      '/events': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        // No rewrite needed, keep /events prefix
        configure: (proxy, options) => {
          proxy.on('error', (err, req, res) => {
            console.error('SSE Proxy error:', err);
            res.writeHead(500, {
              'Content-Type': 'text/plain',
            });
            res.end('SSE proxy error.');
          });
        }
      },
      // Notification service proxy to remote Gateway
      '/notify': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        // No rewrite needed, keep /notify prefix
      },
      // Online users proxy to remote Gateway  
      '/online-users': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        // No rewrite needed, keep /online-users prefix
      },
      // Generic proxy - handle other possible endpoints
      '/ws': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        ws: true, // Enable WebSocket proxying
      }
    },

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

  define: {
    'import.meta.env.DEV': JSON.stringify(process.env.NODE_ENV === 'development')
  },

  build: {
    outDir: 'dist',
    sourcemap: true,
    // Drop console logs in production
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true
      }
    },
    rollupOptions: {
      output: {
        manualChunks: {
          'shiki': ['shiki'],
          'markdown': ['remark', 'remark-gfm', 'remark-breaks', 'remark-html', 'rehype', 'rehype-raw', 'rehype-stringify', 'rehype-highlight', 'rehype-stringify'],
          'vendor': ['vue', 'vue-router', 'pinia', '@vueuse/core'],
          'ui': ['primeicons']
        }
      }
    }
  }
})); 