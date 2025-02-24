const fs = require('fs');

const filePath = 'src/plugins/shiki.js';
let content = fs.readFileSync(filePath, 'utf8');

// 修复getHighlighter函数缺少结束大括号的问题
content = content.replace(
  /(export async function getHighlighter\([^)]*\) \{\s*return createShikiHighlighter\([^)]*\);\s*)\n(\s*\/\/ Resolve language)/,
  '$1}\n\n$2'
);

// 删除多余的大括号
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
      nextLine.match(/^[a-zA-Z_]/) || nextLine.startsWith('//')) {
      console.log(`Removing extra } at line ${i + 1}`);
      continue;
    }
  }

  fixedLines.push(line);
}

fs.writeFileSync(filePath, fixedLines.join('\n'), 'utf8');
console.log('✅ Fixed shiki.js syntax errors'); 