#!/usr/bin/env node

/**
 * 生产环境日志清理脚本
 * 扫描并清理不必要的调试日志
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class LogCleanupTool {
  constructor() {
    this.srcDir = path.join(__dirname, '../src');
    this.stats = {
      filesScanned: 0,
      filesModified: 0,
      logsRemoved: 0,
      patterns: new Map()
    };
    
    // 要清理的日志模式
    this.cleanupPatterns = [
      // 开发调试日志 (以表情符号开头)
      {
        pattern: /console\.log\(['"`][📤📨🌐🔄].*?['"`].*?\);?\s*/g,
        description: '开发调试日志 (表情符号标识)'
      },
      
      // 特定模块的调试日志
      {
        pattern: /console\.log\(['"`].*\[CHAT_STORE\].*?['"`].*?\);?\s*/g,
        description: 'Chat Store 调试日志'
      },
      {
        pattern: /console\.log\(['"`].*\[AUTH\].*?['"`].*?\);?\s*/g,
        description: 'Auth 调试日志'
      },
      {
        pattern: /console\.log\(['"`].*\[DEBUG\].*?['"`].*?\);?\s*/g,
        description: 'Debug 标记的日志'
      },
      {
        pattern: /console\.log\(['"`].*\[PRELOAD\].*?['"`].*?\);?\s*/g,
        description: 'Preload 调试日志'
      },
      {
        pattern: /console\.log\(['"`].*\[FETCH\].*?['"`].*?\);?\s*/g,
        description: 'Fetch 调试日志'
      },
      {
        pattern: /console\.log\(['"`].*\[SSE\].*?['"`].*?\);?\s*/g,
        description: 'SSE 调试日志'
      },
      
      // 临时调试代码
      {
        pattern: /\/\/ TODO: remove.*\n.*console\.\w+\(.*?\);?\s*/g,
        description: 'TODO 标记的临时日志'
      },
      {
        pattern: /\/\/ DEBUG:.*\n.*console\.\w+\(.*?\);?\s*/g,
        description: 'DEBUG 标记的临时日志'
      },
      {
        pattern: /\/\* DEBUG \*\/.*console\.\w+\(.*?\);?\s*/g,
        description: '注释标记的调试日志'
      },
      
      // 性能测试日志
      {
        pattern: /console\.time\(['"`].*?['"`]\);?\s*/g,
        description: 'console.time 性能测试'
      },
      {
        pattern: /console\.timeEnd\(['"`].*?['"`]\);?\s*/g,
        description: 'console.timeEnd 性能测试'
      },
      
      // 多行调试代码块
      {
        pattern: /\/\/ DEV START[\s\S]*?\/\/ DEV END\s*/g,
        description: '开发环境代码块'
      }
    ];
    
    // 要保留的重要日志模式
    this.preservePatterns = [
      /console\.error\(/,
      /console\.warn\(/,
      /errorHandler\.handle\(/,
      /this\.originalConsole\./
    ];
    
    // 要扫描的文件扩展名
    this.extensions = ['.js', '.vue', '.ts'];
  }

  /**
   * 扫描指定目录
   */
  async scanDirectory(dir) {
    const entries = await fs.promises.readdir(dir, { withFileTypes: true });
    
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      
      if (entry.isDirectory()) {
        // 跳过 node_modules 和其他不需要的目录
        if (['node_modules', '.git', 'dist', 'build'].includes(entry.name)) {
          continue;
        }
        await this.scanDirectory(fullPath);
      } else if (entry.isFile()) {
        const ext = path.extname(entry.name);
        if (this.extensions.includes(ext)) {
          await this.processFile(fullPath);
        }
      }
    }
  }

  /**
   * 处理单个文件
   */
  async processFile(filePath) {
    try {
      const content = await fs.promises.readFile(filePath, 'utf-8');
      const originalContent = content;
      let modifiedContent = content;
      let fileModified = false;
      
      this.stats.filesScanned++;
      
      // 应用清理模式
      for (const { pattern, description } of this.cleanupPatterns) {
        const matches = modifiedContent.match(pattern);
        if (matches) {
          // 检查是否包含要保留的模式
          const shouldPreserve = matches.some(match => 
            this.preservePatterns.some(preservePattern => 
              preservePattern.test(match)
            )
          );
          
          if (!shouldPreserve) {
            modifiedContent = modifiedContent.replace(pattern, '');
            const removedCount = matches.length;
            this.stats.logsRemoved += removedCount;
            fileModified = true;
            
            // 统计模式使用
            const currentCount = this.stats.patterns.get(description) || 0;
            this.stats.patterns.set(description, currentCount + removedCount);
            
            console.log(`🧹 ${path.relative(this.srcDir, filePath)}: 移除 ${removedCount} 个 ${description}`);
          }
        }
      }
      
      // 如果文件被修改，写回文件
      if (fileModified) {
        // 清理多余的空行
        modifiedContent = modifiedContent.replace(/\n\s*\n\s*\n/g, '\n\n');
        
        await fs.promises.writeFile(filePath, modifiedContent);
        this.stats.filesModified++;
        console.log(`已更新: ${path.relative(this.srcDir, filePath)}`);
      }
      
    } catch (error) {
      console.error(`ERROR: 处理文件失败: ${filePath}`, error.message);
    }
  }

  /**
   * 生成清理报告
   */
  generateReport() {
    console.log('\n清理报告:');
    console.log('='.repeat(50));
    console.log(`📁 扫描文件: ${this.stats.filesScanned}`);
    console.log(`修改文件: ${this.stats.filesModified}`);
    console.log(`🗑️  移除日志: ${this.stats.logsRemoved}`);
    
    if (this.stats.patterns.size > 0) {
      console.log('\n📋 按类型分类:');
      for (const [pattern, count] of this.stats.patterns.entries()) {
        console.log(`  • ${pattern}: ${count}`);
      }
    }
    
    if (this.stats.logsRemoved > 0) {
      console.log(`\n✨ 成功清理 ${this.stats.logsRemoved} 个调试日志！`);
    } else {
      console.log('\n没有找到需要清理的日志。');
    }
  }

  /**
   * 创建备份
   */
  async createBackup() {
    const backupDir = path.join(__dirname, '../.backup');
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const backupPath = path.join(backupDir, `src-backup-${timestamp}`);
    
    try {
      await fs.promises.mkdir(backupDir, { recursive: true });
      
      // 简单的文件复制备份
      console.log(`💾 创建备份到: ${backupPath}`);
      
      // 这里可以添加更复杂的备份逻辑
      // 为了简单起见，我们只记录备份意图
      const backupInfo = {
        timestamp,
        srcDir: this.srcDir,
        note: 'Log cleanup backup'
      };
      
      await fs.promises.writeFile(
        path.join(backupDir, `backup-info-${timestamp}.json`),
        JSON.stringify(backupInfo, null, 2)
      );
      
      console.log('备份信息已保存');
    } catch (error) {
      console.warn('WARNING: 备份创建失败:', error.message);
    }
  }

  /**
   * 运行清理
   */
  async run(options = {}) {
    const { dryRun = false, backup = true } = options;
    
    console.log('🧹 开始清理生产环境调试日志...\n');
    
    if (dryRun) {
      console.log('DRY RUN 模式 - 不会修改文件\n');
    } else if (backup) {
      await this.createBackup();
    }
    
    await this.scanDirectory(this.srcDir);
    
    this.generateReport();
    
    if (dryRun) {
      console.log('\n这是试运行模式。要真正清理，请运行: npm run cleanup-logs');
    }
  }
}

// 命令行参数处理
const args = process.argv.slice(2);
const dryRun = args.includes('--dry-run') || args.includes('-d');
const noBackup = args.includes('--no-backup');

// 运行清理
const cleaner = new LogCleanupTool();
cleaner.run({ 
  dryRun, 
  backup: !noBackup 
}).catch(console.error);