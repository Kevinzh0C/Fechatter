const fs = require('fs');
const path = require('path');

function carefulConsoleWrap(filePath) {
  let content = fs.readFileSync(filePath, 'utf8');

  // 只处理没有被包装的console语句
  // 匹配模式：行首的console.log/warn/error等，但不在if语句内
  const consoleRegex = /^(\s*)(console\.(log|warn|error|info|debug|trace)\([^)]*\);?)$/gm;

  const wrappedContent = content.replace(consoleRegex, (match, indent, consoleStatement) => {
    // 检查是否已经被包装
    const lines = content.split('\n');
    const matchIndex = content.indexOf(match);
    const linesBefore = content.substring(0, matchIndex).split('\n');
    const currentLineIndex = linesBefore.length - 1;

    // 检查前一行是否已经有if (import.meta.env.DEV)
    if (currentLineIndex > 0) {
      const prevLine = lines[currentLineIndex - 1];
      if (prevLine.includes('import.meta.env.DEV')) {
        return match; // 已经被包装，跳过
      }
    }

    // 包装console语句
    return `${indent}if (import.meta.env.DEV) {\n${indent}  ${consoleStatement}\n${indent}}`;
  });

  return wrappedContent;
}

// 需要修复的文件列表
const filesToFix = [
  'src/utils/errorHandler.js',
  'src/router/index.js',
  'src/services/sse-minimal.js',
  'src/stores/chat.js',
  'src/utils/performanceMonitor.js'
];

console.log('🔧 Starting careful console.log wrapping...\n');

filesToFix.forEach(file => {
  try {
    if (fs.existsSync(file)) {
      console.log(`Processing ${file}...`);
      const originalContent = fs.readFileSync(file, 'utf8');
      const fixedContent = carefulConsoleWrap(file);

      // 只有内容真的变化了才写入
      if (originalContent !== fixedContent) {
        fs.writeFileSync(file, fixedContent, 'utf8');
        console.log(`✅ Updated ${file}`);
      } else {
        console.log(`⚡ ${file} already fixed`);
      }
    } else {
      console.log(`⚠️  File not found: ${file}`);
    }
  } catch (error) {
    console.error(`❌ Error processing ${file}:`, error.message);
  }
});

console.log('\n🎉 Careful console wrapping complete!'); 