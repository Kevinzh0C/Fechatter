import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";
import viteShikiPlugin from "./vite-plugin-shiki.js";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    viteShikiPlugin({
      theme: 'dark',
      lineNumbers: true,
      cache: true
    })
  ],

  // Path aliases
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,

    // 关键：添加代理来解决CORS问题 - 统一通过Gateway
    proxy: {
      // API代理到Gateway (统一入口)
      '/api': {
        target: 'http://localhost:8080', // Gateway地址
        changeOrigin: true,
        // 不需要rewrite，保持/api前缀
      },
      // 文件服务代理到Gateway  
      '/files': {
        target: 'http://localhost:8080', // Gateway地址
        changeOrigin: true,
        // 不需要rewrite，保持/files前缀
      },
      // SSE事件代理到Gateway
      '/events': {
        target: 'http://localhost:8080', // Gateway地址 
        changeOrigin: true,
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
      // 保留旧的sse-proxy以兼容（但现在也指向Gateway）
      '/sse-proxy': {
        target: 'http://localhost:8080', // Gateway地址，不再直接连接notify_server
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/sse-proxy/, ''), // 移除代理前缀
        configure: (proxy, options) => {
          proxy.on('error', (err, req, res) => {
            console.error('Proxy error:', err);
            res.writeHead(500, {
              'Content-Type': 'text/plain',
            });
            res.end('Something went wrong with the proxy.');
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
}));
