#!/usr/bin/env node

/**
 * Fechatter API Contract Automatic Fixer
 * 自动修复前后端API路径不匹配问题
 */

const fs = require('fs');
const path = require('path');

// API路径修复规则
const fixes = [
  {
    name: '搜索API路径和方法修复',
    files: ['fechatter_frontend/src/stores/chat.js'],
    patterns: [
      {
        from: /axios\.get\(\`\/api\/chat\/\$\{chatId\}\/search\`/g,
        to: 'axios.post(`/api/chat/${chatId}/messages/search`',
        description: '修复搜索API路径和HTTP方法'
      }
    ]
  },
  
  {
    name: '工作空间API路径修复',
    files: ['fechatter_frontend/src/stores/workspace.js'],
    patterns: [
      {
        from: /axios\.get\('\/api\/workspace'\)/g,
        to: "axios.get('/api/workspaces')",
        description: '修复工作空间获取API路径'
      }
    ]
  },
  
  {
    name: '移除不必要的文件API调用',
    files: ['fechatter_frontend/src/stores/chat.js'],
    patterns: [
      {
        from: /async getFileUrl\(filePath\) \{\s*try \{\s*const response = await axios\.get\(\`\/api\/files\/\$\{filePath\}\`\);\s*return response\.data;\s*\} catch \(error\) \{\s*console\.error\('Failed to get file URL:', error\);\s*return null;\s*\}\s*\},/g,
        to: '// getFileUrl moved to component level - files are served directly by backend',
        description: '移除多余的getFileUrl方法，文件直接由后端提供'
      }
    ]
  }
];

// 实现缺失的后端API路由
const missingBackendAPIs = [
  {
    name: '工作空间邀请API',
    path: '/api/workspace/invite',
    method: 'POST',
    description: '邀请用户到工作空间',
    implementation: `
// 在后端添加邀请路由 (lib.rs中auth_routes部分)
.route("/workspace/invite", post(invite_user_to_workspace_handler))

// 创建对应的handler
pub async fn invite_user_to_workspace_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserClaims>,
    Json(payload): Json<InviteUserPayload>
) -> Result<Json<ApiResponse>, AppError> {
    // 实现邀请逻辑
}
`
  },
  
  {
    name: '移除工作空间用户API',
    path: '/api/workspace/users/{userId}',
    method: 'DELETE',
    description: '从工作空间移除用户',
    implementation: `
// 在后端添加移除用户路由
.route("/workspace/users/{user_id}", delete(remove_user_from_workspace_handler))

// 创建对应的handler
pub async fn remove_user_from_workspace_handler(
    State(state): State<AppState>,
    Extension(user): Extension<UserClaims>,
    Path(user_id): Path<i64>
) -> Result<Json<ApiResponse>, AppError> {
    // 实现移除用户逻辑
}
`
  }
];

// 应用修复
function applyFixes() {
  console.log('🔧 Fechatter API Contract Auto-Fixer');
  console.log('=' .repeat(50));
  
  let totalChanges = 0;
  
  fixes.forEach(fix => {
    console.log(`\n📝 ${fix.name}:`);
    
    fix.files.forEach(filePath => {
      if (!fs.existsSync(filePath)) {
        console.log(`   ⚠️  文件不存在: ${filePath}`);
        return;
      }
      
      let content = fs.readFileSync(filePath, 'utf8');
      let fileChanged = false;
      
      fix.patterns.forEach(pattern => {
        const matches = content.match(pattern.from);
        if (matches) {
          content = content.replace(pattern.from, pattern.to);
          console.log(`   ✅ ${pattern.description} (${matches.length} 处修改)`);
          fileChanged = true;
          totalChanges += matches.length;
        }
      });
      
      if (fileChanged) {
        // 创建备份
        const backupPath = `${filePath}.backup.${Date.now()}`;
        fs.writeFileSync(backupPath, fs.readFileSync(filePath));
        
        // 写入修复后的内容
        fs.writeFileSync(filePath, content);
        console.log(`   💾 已保存修改: ${filePath}`);
        console.log(`   📄 备份创建: ${backupPath}`);
      }
    });
  });
  
  return totalChanges;
}

// 生成缺失API实现指南
function generateMissingAPIGuide() {
  const guide = `# 缺失API实现指南

## 🚨 需要在后端实现的API

${missingBackendAPIs.map(api => `### ${api.name}
- **路径**: \`${api.path}\`
- **方法**: \`${api.method}\`
- **描述**: ${api.description}

**实现代码**:
\`\`\`rust${api.implementation}
\`\`\`

`).join('')}

## 📋 实现步骤

1. **更新路由定义** (fechatter_server/src/lib.rs)
   - 在适当的路由组中添加新的路由定义
   - 确保middleware配置正确

2. **创建Handler函数**
   - 在 fechatter_server/src/handlers/ 下创建对应的handler
   - 实现业务逻辑和数据验证

3. **添加数据结构**
   - 在需要的地方定义请求/响应的数据结构
   - 使用serde进行序列化

4. **测试API**
   - 编写单元测试
   - 进行集成测试
   - 验证前后端整合

## 🔄 API契约维护

### 自动化检查
\`\`\`bash
# 运行契约检查
node api-contract-checker.js

# 自动修复已知问题
node api-contract-fixer.js
\`\`\`

### Git Hooks建议
\`\`\`bash
# pre-commit hook
#!/bin/sh
node api-contract-checker.js
if [ $? -ne 0 ]; then
  echo "❌ API契约检查失败，请修复不匹配问题后再提交"
  exit 1
fi
\`\`\`

---
*生成时间: ${new Date().toISOString()}*
`;

  fs.writeFileSync('MISSING_API_GUIDE.md', guide);
  console.log('\n📋 缺失API实现指南已生成: MISSING_API_GUIDE.md');
}

// 验证修复结果
function validateFixes() {
  console.log('\n🔍 验证修复结果...');
  
  try {
    const checker = require('./api-contract-checker.js');
    const results = checker.checkAPIAlignment();
    
    console.log(`✅ 修复后对应率: ${results.alignmentScore.toFixed(1)}%`);
    
    if (results.missing_in_backend.length === 0) {
      console.log('🎉 所有前端API调用都有对应的后端支持！');
    } else {
      console.log(`⚠️  仍有 ${results.missing_in_backend.length} 个API需要后端实现`);
    }
    
    return results;
  } catch (error) {
    console.log('❌ 验证过程中出错:', error.message);
    return null;
  }
}

// 创建自动化脚本
function createAutomationScripts() {
  // Pre-commit hook
  const preCommitHook = `#!/bin/sh
# Fechatter API Contract Pre-commit Hook

echo "🔍 检查API契约..."
node api-contract-checker.js

if [ $? -ne 0 ]; then
  echo ""
  echo "❌ API契约检查失败！"
  echo "   请运行 'node api-contract-fixer.js' 修复已知问题"
  echo "   或手动修复不匹配的API路径"
  echo ""
  exit 1
fi

echo "✅ API契约检查通过"
`;

  // Package.json scripts
  const packageScripts = {
    "api:check": "node api-contract-checker.js",
    "api:fix": "node api-contract-fixer.js",
    "api:validate": "node api-contract-checker.js && echo '✅ API契约验证通过'"
  };

  fs.writeFileSync('.git/hooks/pre-commit', preCommitHook);
  fs.chmodSync('.git/hooks/pre-commit', '755');
  
  console.log('\n🛠️  自动化脚本已创建:');
  console.log('   📄 Git pre-commit hook: .git/hooks/pre-commit');
  console.log('   📦 推荐的package.json scripts:');
  console.log(JSON.stringify(packageScripts, null, 2));
}

// 主函数
function main() {
  const totalChanges = applyFixes();
  
  console.log(`\n📊 修复总结:`);
  console.log(`   总共修复: ${totalChanges} 处`);
  
  if (totalChanges > 0) {
    console.log('\n🔄 重新验证API契约...');
    const results = validateFixes();
    
    if (results) {
      console.log(`   修复前后对比:`);
      console.log(`   - 匹配改进: 提升到 ${results.alignmentScore.toFixed(1)}%`);
    }
  }
  
  generateMissingAPIGuide();
  
  try {
    createAutomationScripts();
  } catch (error) {
    console.log('⚠️  无法创建Git hooks (可能不在Git仓库中)');
  }
  
  console.log('\n🎯 下一步行动:');
  console.log('   1. 检查备份文件，确认修改正确');
  console.log('   2. 根据 MISSING_API_GUIDE.md 实现缺失的后端API');
  console.log('   3. 运行 node api-contract-checker.js 验证最终结果');
  console.log('   4. 测试应用功能，确保修复生效');
}

if (require.main === module) {
  main();
}

module.exports = {
  applyFixes,
  generateMissingAPIGuide,
  validateFixes
}; 