#!/usr/bin/env node

const { spawn } = require('child_process');

console.log('\n启动 Fechatter 开发服务器...');
console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');

// 先运行配置复制
const copyConfigs = spawn('node', ['scripts/copy-configs.js'], {
  stdio: 'inherit'
});

copyConfigs.on('close', (code) => {
  if (code === 0) {
    console.log('\n开发服务器启动中...');
    console.log('📱 请在浏览器中访问以下地址:\n');
    console.log('   🌐 本地地址: \x1b[4m\x1b[36mhttp://localhost:5173\x1b[0m');
    console.log('   🔗 网络地址: 请看下方 Vite 输出中的 Network 地址\n');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('按 Ctrl+C 停止服务器');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    
    // 启动Vite
    const vite = spawn('vite', [], {
      stdio: 'inherit'
    });
    
    process.on('SIGINT', () => {
      console.log('\n�� 正在停止开发服务器...');
      vite.kill('SIGINT');
    });
  }
});
