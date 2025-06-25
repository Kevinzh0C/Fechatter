import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";
// import viteShikiPlugin from "./vite-plugin-shiki.js";
import { fileURLToPath, URL } from 'node:url'
import VitePluginVueDevTools from 'vite-plugin-vue-devtools'

const host = process.env.TAURI_DEV_HOST;

// 🔐 SECURITY: Function to sanitize sensitive data in URLs
function sanitizeUrl(url) {
  return url
    .replace(/access_token=([^&]+)/g, (match, token) => `access_token=${token.substring(0, 8)}***`)
    .replace(/token=([^&]+)/g, (match, token) => `token=${token.substring(0, 8)}***`)
    .replace(/password=([^&]+)/g, 'password=***')
    .replace(/secret=([^&]+)/g, 'secret=***')
    .replace(/key=([^&]+)/g, 'key=***');
}

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

  // Optimize dependencies - 移除Shiki相关配置
  optimizeDeps: {
    include: ['highlight.js'],
    exclude: ['@tauri-apps/api']
  },

  // Worker支持
  worker: {
    format: 'es'
  },

  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    open: false, // 🚀 CHANGED: 不自动打开浏览器，显示URL供手动打开
    cors: true,
    headers: {
      'Cross-Origin-Embedder-Policy': 'require-corp',
      'Cross-Origin-Opener-Policy': 'same-origin',
    },
    fs: {
      allow: ['..']
    },

    // 基本代理配置 - 移除复杂的WASM处理
    proxy: {
      // 🤖 PRIORITY: Bot API代理直接到远程Gateway (新的bot-server已部署)
      '/api/bot': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        timeout: 5000,
        configure: (proxy, options) => {
          proxy.on('error', (err, req, res) => {
            console.error('🚨 Bot API Proxy error:', err.message);
            if (!res.headersSent) {
              res.writeHead(503, {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*',
                'Access-Control-Allow-Headers': '*',
                'Access-Control-Allow-Methods': '*'
              });
              res.end(JSON.stringify({
                error: 'Bot service temporarily unavailable',
                code: 'SERVICE_UNAVAILABLE',
                development: true
              }));
            }
          });
          proxy.on('proxyReq', (proxyReq, req, res) => {
            console.log(`🤖 [Proxy] Bot API: ${req.method} ${sanitizeUrl(req.url)} → http://45.77.178.85:8080`);
          });
        }
      },

      // API代理到远程Gateway (统一入口) - FIXED: 统一指向远程Gateway
      '/api': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        timeout: 5000,
        // 不需要rewrite，保持/api前缀
        configure: (proxy, options) => {
          proxy.on('error', (err, req, res) => {
            console.error('🚨 API Proxy error:', err.message);
            if (!res.headersSent) {
              res.writeHead(503, {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*',
                'Access-Control-Allow-Headers': '*',
                'Access-Control-Allow-Methods': '*'
              });
              res.end(JSON.stringify({
                error: 'Backend service temporarily unavailable',
                code: 'SERVICE_UNAVAILABLE',
                development: true,
                message: 'This is normal in development when remote services are down'
              }));
            }
          });
          proxy.on('proxyReq', (proxyReq, req, res) => {
            if (!req.url.startsWith('/api/bot')) {
              console.log(`🌐 [Proxy] General API: ${req.method} ${sanitizeUrl(req.url)} → http://45.77.178.85:8080`);
            }
          });
        }
      },
      // Health check proxy
      '/health': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        timeout: 3000,
        configure: (proxy, options) => {
          proxy.on('error', (err, req, res) => {
            if (!res.headersSent) {
              res.writeHead(200, {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*'
              });
              res.end(JSON.stringify({
                status: 'development',
                message: 'Health check unavailable in development',
                timestamp: new Date().toISOString()
              }));
            }
          });
        }
      },
      // 文件服务代理到远程Gateway  
      '/files': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        timeout: 10000,
        // 不需要rewrite，保持/files前缀
        configure: (proxy, options) => {
          proxy.on('error', (err, req, res) => {
            console.error('🚨 Files Proxy error:', err.message);
            if (!res.headersSent) {
              res.writeHead(503, {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*'
              });
              res.end(JSON.stringify({
                error: 'File service temporarily unavailable',
                code: 'SERVICE_UNAVAILABLE'
              }));
            }
          });
        }
      },
      // SSE事件代理到远程Gateway - 🚀 ENHANCED: Optimized for EventSource
      '/events': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        timeout: 0, // 🚀 CRITICAL: No timeout for SSE connections
        ws: false,
        // 🚀 CRITICAL: Disable response buffering for SSE
        buffer: false,
        // 不需要rewrite，保持/events前缀
        configure: (proxy, options) => {
          proxy.on('proxyReq', (proxyReq, req, res) => {
            // 🔐 SECURITY: Sanitize sensitive information in logs
            console.log(`📡 [Proxy] SSE: ${req.method} ${sanitizeUrl(req.url)} → http://45.77.178.85:8080`);
            // 🚀 CRITICAL: Set proper headers for SSE
            proxyReq.setHeader('Accept', 'text/event-stream');
            proxyReq.setHeader('Cache-Control', 'no-cache');
            proxyReq.setHeader('Connection', 'keep-alive');
            // 🚀 CRITICAL: Remove any buffering headers
            proxyReq.removeHeader('content-length');
          });
          proxy.on('proxyRes', (proxyRes, req, res) => {
            console.log(`📡 [Proxy] SSE Response: ${proxyRes.statusCode} - ${proxyRes.headers['content-type']}`);
            // 🚀 CRITICAL: Ensure SSE headers and streaming are preserved
            if (proxyRes.headers['content-type']?.includes('text/event-stream')) {
              // Remove content-length to enable streaming
              delete proxyRes.headers['content-length'];
              // Set SSE headers
              proxyRes.headers['cache-control'] = 'no-cache';
              proxyRes.headers['connection'] = 'keep-alive';
              proxyRes.headers['access-control-allow-origin'] = '*';
              proxyRes.headers['access-control-allow-credentials'] = 'true';
              console.log(`📡 [Proxy] SSE Headers configured for streaming`);
            }
          });
          proxy.on('error', (err, req, res) => {
            console.error('🚨 SSE Proxy error:', err.message);
            console.error('🚨 SSE Error details:', { url: sanitizeUrl(req.url), method: req.method });
            // 🚀 CRITICAL: Don't interfere with SSE connections, let them fail naturally
            if (!res.headersSent) {
              res.writeHead(503, {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*'
              });
              res.end(JSON.stringify({
                error: 'SSE service temporarily unavailable',
                code: 'SSE_PROXY_ERROR',
                development: true
              }));
            }
          });
        }
      },
      // 通知服务代理到远程Gateway
      '/notify': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        timeout: 5000,
        // 不需要rewrite，保持/notify前缀
        configure: (proxy, options) => {
          proxy.on('error', (err, req, res) => {
            if (!res.headersSent) {
              res.writeHead(503, {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*'
              });
              res.end(JSON.stringify({
                error: 'Notification service temporarily unavailable',
                code: 'SERVICE_UNAVAILABLE'
              }));
            }
          });
        }
      },
      // 在线用户代理到远程Gateway  
      '/online-users': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        timeout: 5000,
        // 不需要rewrite，保持/online-users前缀
        configure: (proxy, options) => {
          proxy.on('error', (err, req, res) => {
            if (!res.headersSent) {
              res.writeHead(200, {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*'
              });
              res.end(JSON.stringify({
                online_users: [],
                total: 0,
                message: 'Online users service unavailable in development'
              }));
            }
          });
        }
      },
      // 通用代理 - 处理其他可能的端点
      '/ws': {
        target: 'http://45.77.178.85:8080',
        changeOrigin: true,
        secure: false,
        ws: true, // Enable WebSocket proxying
        timeout: 5000,
        configure: (proxy, options) => {
          proxy.on('error', (err, req, res) => {
            console.error('🚨 WebSocket Proxy error:', err.message);
            // WebSocket errors are handled differently
          });
        }
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
  // assetsInclude: ['**/*.wasm'],

  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // 优化依赖分块 - 移除Shiki，添加highlight.js
          'highlight-chunk': ['highlight.js'],
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