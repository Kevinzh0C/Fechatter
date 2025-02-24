import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";
// import viteShikiPlugin from "./vite-plugin-shiki.js";
import { fileURLToPath, URL } from 'node:url'
import VitePluginVueDevTools from 'vite-plugin-vue-devtools'

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue({
      script: {
        defineModel: true,
        propsDestructure: true
      }
    }),
    // viteShikiPlugin({
    //   theme: 'dark',
    //   lineNumbers: true,
    //   cache: true
    // }),
    // VitePluginVueDevTools() // 🔧 临时禁用解决格式化器卡死问题
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

  // Optimize dependencies including WASM-based packages
  optimizeDeps: {
    include: ['shiki'],
    exclude: ['shiki/wasm']
  },

  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    open: true,
    cors: true,
    headers: {
      'Cross-Origin-Embedder-Policy': 'require-corp',
      'Cross-Origin-Opener-Policy': 'same-origin',
    },
    fs: {
      allow: ['..']
    },

    // Complete proxy configuration to solve CORS issues - unified through remote Gateway
    proxy: {
      // 🤖 PRIORITY: Bot API代理直接到远程Gateway (新的bot-server已部署)
      '/api/bot': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        configure: (proxy, options) => {
          proxy.on('error', (err, req, res) => {
            console.error('🚨 Bot API Proxy error:', err.message);
          });
          proxy.on('proxyReq', (proxyReq, req, res) => {
            console.log(`🤖 [Proxy] Bot API: ${req.method} ${req.url} → http://45.77.178.85:8080`);
          });
        }
      },

      // API代理到远程Gateway (统一入口) - FIXED: 统一指向远程Gateway
      '/api': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        // 不需要rewrite，保持/api前缀
        configure: (proxy, options) => {
          proxy.on('proxyReq', (proxyReq, req, res) => {
            if (!req.url.startsWith('/api/bot')) {
              console.log(`🌐 [Proxy] General API: ${req.method} ${req.url} → http://45.77.178.85:8080`);
            }
          });
        }
      },
      // Health check proxy
      '/health': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
      },
      // 文件服务代理到远程Gateway  
      '/files': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        // 不需要rewrite，保持/files前缀
      },
      // SSE事件代理到远程Gateway
      '/events': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        // 不需要rewrite，保持/events前缀
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
      // 通知服务代理到远程Gateway
      '/notify': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        // 不需要rewrite，保持/notify前缀
      },
      // 在线用户代理到远程Gateway  
      '/online-users': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        // 不需要rewrite，保持/online-users前缀
      },
      // 通用代理 - 处理其他可能的端点
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

  // Include WASM files as assets
  assetsInclude: ['**/*.wasm'],

  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // Separate large dependencies to reduce warnings
          'shiki-chunk': ['shiki'],
          'vue-chunk': ['vue', 'vue-router', 'pinia'],
          'ui-chunk': ['@headlessui/vue', '@heroicons/vue']
        }
      }
    },
    chunkSizeWarningLimit: 1000,
    target: 'esnext',
    outDir: 'dist',
    sourcemap: true
  },

  // Suppress specific warnings
  logLevel: 'warn',
  esbuild: {
    logOverride: {
      'this-is-undefined-in-esm': 'silent',
      'import-is-undefined': 'silent'
    }
  }
}); 