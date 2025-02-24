const fs = require('fs');
const { execSync } = require('child_process');

// 获取所有包含console的文件
function getAllConsoleFiles() {
  try {
    const result = execSync('grep -r "console\\." src/ --include="*.js" --include="*.vue" -l', { encoding: 'utf8' });
    return result.trim().split('\n').filter(file => file.length > 0);
  } catch (error) {
    console.log('No console files found or error:', error.message);
    return [];
  }
}

// 修复console语法错误
function fixConsoleSyntax(filePath) {
  let content = fs.readFileSync(filePath, 'utf8');
  let modified = false;

  // 修复模式1: console.error('message', {\n}\n  property: value
  content = content.replace(
    /(console\.(log|error|warn|info)\([^{]+\{\s*\n\s*\}\s*\n\s*)([a-zA-Z_][a-zA-Z0-9_]*:\s*[^,\n]+)/g,
    (match, consoleStart, method, property) => {
      modified = true;
      return consoleStart.replace(/\{\s*\n\s*\}\s*\n\s*/, '{\n        ') + property;
    }
  );

  // 修复模式2: 删除多余的单独的 }
  const lines = content.split('\n');
  const fixedLines = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // 如果是单独的 }，检查上下文
    if (trimmed === '}' && i > 0 && i < lines.length - 1) {
      const prevLine = lines[i - 1]?.trim() || '';
      const nextLine = lines[i + 1]?.trim() || '';

      // 如果前一行已经有 } 或者下一行开始新的语句，这个 } 可能是多余的
      if (prevLine.endsWith('}') || prevLine.endsWith('});') ||
        nextLine.match(/^[a-zA-Z_]/)) {
        console.log(`Removing extra } at line ${i + 1} in ${filePath}`);
        modified = true;
        continue;
      }
    }

    fixedLines.push(line);
  }

  if (modified) {
    fs.writeFileSync(filePath, fixedLines.join('\n'), 'utf8');
    return true;
  }

  return false;
}

// 主函数
function main() {
  console.log('🔧 Starting comprehensive console syntax fix...\n');

  const files = getAllConsoleFiles();
  console.log(`Found ${files.length} files with console statements\n`);

  let fixedCount = 0;

  files.forEach(file => {
    try {
      if (fs.existsSync(file)) {
        console.log(`Processing ${file}...`);
        if (fixConsoleSyntax(file)) {
          console.log(`✅ Fixed ${file}`);
          fixedCount++;
        } else {
          console.log(`⚡ ${file} - no fixes needed`);
        }
      }
    } catch (error) {
      console.error(`❌ Error processing ${file}:`, error.message);
    }
  });

  console.log(`\n🎉 Fixed ${fixedCount} files!`);
}

main(); 