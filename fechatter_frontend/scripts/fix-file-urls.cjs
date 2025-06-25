#!/usr/bin/env node

/**
 * 🔧 自动修复所有组件中的文件URL处理
 */

const fs = require('fs');
const path = require('path');

const componentsToFix = [
  'src/components/discord/DiscordMessageItem.vue',
  'src/components/chat/FilePreview.vue',
  'src/components/common/EnhancedImageModal.vue'
];

function fixComponent(filePath) {
  try {
    let content = fs.readFileSync(filePath, 'utf8');
    
    // 添加import语句（如果不存在）
    if (!content.includes('getStandardFileUrl')) {
      const importSection = content.match(/(import.*from.*['"]@\/.*['"];?\n)+/);
      if (importSection) {
        const newImport = "import { getStandardFileUrl } from '@/utils/fileUrlHandler';\n";
        content = content.replace(importSection[0], importSection[0] + newImport);
      }
    }
    
    // 替换getFileUrl函数
    const getFileUrlPattern = /const getFileUrl = \([^)]*\) => \{[\s\S]*?\n\}/;
    const newGetFileUrl = `const getFileUrl = (file) => {
  // 🔧 Use unified file URL handler to automatically handle all formats
  return getStandardFileUrl(file, {
    workspaceId: props.message?.workspace_id || props.workspaceId
  });
}`;
    
    if (getFileUrlPattern.test(content)) {
      content = content.replace(getFileUrlPattern, newGetFileUrl);
      fs.writeFileSync(filePath, content);
      console.log(`✅ Fixed: ${filePath}`);
      return true;
    } else {
      console.log(`⚠️  No getFileUrl function found in: ${filePath}`);
      return false;
    }
    
  } catch (error) {
    console.error(`❌ Error fixing ${filePath}:`, error.message);
    return false;
  }
}

// 主要执行函数
function main() {
  console.log('🔧 Starting automatic file URL fix...\n');
  
  let fixedCount = 0;
  let totalCount = 0;
  
  for (const componentPath of componentsToFix) {
    totalCount++;
    if (fixComponent(componentPath)) {
      fixedCount++;
    }
  }
  
  console.log(`\n📊 Fix Summary:`);
  console.log(`✅ Fixed: ${fixedCount}/${totalCount} files`);
  console.log(`🎯 Success Rate: ${((fixedCount/totalCount)*100).toFixed(1)}%`);
  
  if (fixedCount === totalCount) {
    console.log(`\n🎉 All file URL handlers successfully unified!`);
    console.log(`📱 Test the fix: http://localhost:5173/file-url-auto-fix-test.html`);
  } else {
    console.log(`\n⚠️  Some files need manual fixing.`);
  }
}

main();
