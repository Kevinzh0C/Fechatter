#!/usr/bin/env node

import { spawn } from 'child_process';
import chalk from 'chalk';

console.log(chalk.cyan('启动 Fechatter 开发服务器...'));
console.log(chalk.gray('📋 复制配置文件中...'));

// 先运行配置复制
const copyConfigs = spawn('node', ['scripts/copy-configs.js'], {
  stdio: 'inherit',
  cwd: process.cwd()
});

copyConfigs.on('close', (code) => {
  if (code === 0) {
    console.log(chalk.green('配置文件复制完成'));
    console.log(chalk.cyan('启动 Vite 开发服务器...'));

    // 启动Vite服务器
    const vite = spawn('vite', ['--host'], {
      stdio: 'pipe',
      cwd: process.cwd()
    });

    let serverStarted = false;

    vite.stdout.on('data', (data) => {
      const output = data.toString();

      // 捕获服务器启动信息
      if (output.includes('Local:') && !serverStarted) {
        serverStarted = true;

        console.log('\n' + chalk.green('🎉 开发服务器启动成功!'));
        console.log(chalk.yellow('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'));
        console.log(chalk.bold.green('📱 Fechatter 前端开发服务器'));
        console.log(chalk.yellow('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'));
        console.log(chalk.cyan('🌐 本地访问地址:'));
        console.log(chalk.white('   ') + chalk.underline.blue('http://localhost:5173'));
        console.log(chalk.cyan('🔗 网络访问地址:'));
        console.log(chalk.white('   ') + chalk.underline.blue('http://192.168.x.x:5173'));
        console.log(chalk.yellow('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'));
        console.log(chalk.green('请复制上面的 URL 到浏览器中打开'));
        console.log(chalk.gray('按 Ctrl+C 停止服务器'));
        console.log(chalk.yellow('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n'));
      } else {
        // 显示其他Vite输出
        process.stdout.write(output);
      }
    });

    vite.stderr.on('data', (data) => {
      // 过滤一些常见的警告
      const output = data.toString();
      if (!output.includes('DeprecationWarning') &&
        !output.includes('ExperimentalWarning')) {
        process.stderr.write(output);
      }
    });

    vite.on('close', (code) => {
      if (code !== 0) {
        console.log(chalk.red(`ERROR: Vite 服务器异常退出，代码: ${code}`));
      } else {
        console.log(chalk.yellow('👋 开发服务器已停止'));
      }
    });

    // 处理进程退出
    process.on('SIGINT', () => {
      console.log(chalk.yellow('\n🛑 正在停止开发服务器...'));
      vite.kill('SIGINT');
    });

    process.on('SIGTERM', () => {
      vite.kill('SIGTERM');
    });

  } else {
    console.log(chalk.red(`ERROR: 配置复制失败，代码: ${code}`));
    process.exit(1);
  }
});

copyConfigs.on('error', (err) => {
  console.log(chalk.red('ERROR: 启动失败:'), err.message);
  process.exit(1);
}); 