#!/usr/bin/env node

/**
 * Fechatter API Contract Checker
 * 检查前后端API路径对应关系和数据格式一致性
 */

const fs = require('fs');
const path = require('path');

// 前端API调用分析
const frontendAPIs = {
  // 认证相关
  auth: {
    '/signin': { method: 'POST', file: 'stores/auth.js', params: ['email', 'password'] },
    '/signup': { method: 'POST', file: 'stores/auth.js', params: ['fullname', 'email', 'password', 'workspace'] },
    '/logout': { method: 'POST', file: 'stores/auth.js', params: [] },
    '/logout_all': { method: 'POST', file: 'stores/auth.js', params: [] },
    '/refresh': { method: 'POST', file: 'stores/auth.js', params: [] },
  },
  
  // 聊天相关
  chat: {
    '/chat': { 
      methods: ['GET', 'POST'], 
      file: 'stores/chat.js', 
      params: {
        GET: [],
        POST: ['name', 'chat_type', 'is_public', 'workspace_id']
      }
    },
    '/chat/{id}': { method: 'DELETE', file: 'stores/chat.js', params: ['id'] },
    '/chat/{id}/messages': { 
      methods: ['GET', 'POST'], 
      file: 'stores/chat.js',
      params: {
        GET: ['id', 'limit', 'offset'],
        POST: ['id', 'content', 'message_type']
      }
    },
    '/chat/{id}/members': { 
      methods: ['GET', 'POST', 'DELETE'], 
      file: 'stores/chat.js',
      params: {
        GET: ['id'],
        POST: ['id', 'member_ids'],
        DELETE: ['id', 'member_ids']
      }
    },
    '/chat/{id}/messages/search': { method: 'POST', file: 'stores/chat.js', params: ['id', 'query'] },
  },
  
  // 文件上传相关
  file: {
    '/upload': { method: 'POST', file: 'stores/chat.js', params: ['file'] },
    '/fix-files/{ws_id}': { method: 'POST', file: 'stores/chat.js', params: ['ws_id'] },
  },
  
  // 工作空间相关
  workspace: {
    '/workspaces': { method: 'GET', file: 'stores/workspace.js', params: [] },
    '/users': { method: 'GET', file: 'stores/workspace.js', params: [] },
    '/workspace/invite': { method: 'POST', file: 'stores/workspace.js', params: ['email', 'role'] },
    '/workspace/users/{userId}': { method: 'DELETE', file: 'stores/workspace.js', params: ['userId'] },
  }
};

// 后端路由定义（从lib.rs分析得出）
const backendRoutes = {
  public: [
    { path: '/signin', method: 'POST' },
    { path: '/signup', method: 'POST' },
    { path: '/refresh', method: 'POST' },
  ],
  
  auth: [
    { path: '/upload', method: 'POST' },
    { path: '/files/{ws_id}/{*path}', method: 'GET' },
    { path: '/fix-files/{ws_id}', method: 'POST' },
    { path: '/users', method: 'GET' },
    { path: '/workspaces', method: 'GET' },
    { path: '/user/switch-workspace', method: 'POST' },
    { path: '/logout', method: 'POST' },
    { path: '/logout_all', method: 'POST' },
  ],
  
  chat_create: [
    { path: '/chat', method: 'POST' },
    { path: '/chat', method: 'GET' },
  ],
  
  chat_manage: [
    { path: '/chat/{id}', method: 'PATCH' },
    { path: '/chat/{id}', method: 'DELETE' },
    { path: '/chat/{id}/members', method: 'GET' },
    { path: '/chat/{id}/members', method: 'POST' },
    { path: '/chat/{id}/members', method: 'DELETE' },
    { path: '/chat/{id}/members/{member_id}', method: 'PATCH' },
    { path: '/chat/{id}/messages', method: 'GET' },
    { path: '/chat/{id}/messages', method: 'POST' },
    { path: '/chat/{id}/messages/search', method: 'POST' },
  ]
};

// 展平后端路由
function flattenBackendRoutes() {
  const flattened = [];
  Object.values(backendRoutes).forEach(routes => {
    flattened.push(...routes);
  });
  return flattened;
}

// 展平前端API
function flattenFrontendAPIs() {
  const flattened = [];
  
  Object.values(frontendAPIs).forEach(apiGroup => {
    Object.entries(apiGroup).forEach(([path, config]) => {
      if (config.methods) {
        // 多个方法
        config.methods.forEach(method => {
          flattened.push({
            path,
            method,
            file: config.file,
            params: config.params[method] || config.params
          });
        });
      } else {
        // 单个方法
        flattened.push({
          path,
          method: config.method,
          file: config.file,
          params: config.params
        });
      }
    });
  });
  
  return flattened;
}

// 路径标准化（处理参数）
function normalizePath(path) {
  return path
    .replace(/\{[^}]+\}/g, '{param}')  // 将所有参数替换为通用形式
    .replace(/\/\*path$/, '/{param}'); // 处理通配符路径
}

// 检查API对应关系
function checkAPIAlignment() {
  console.log('🔍 Fechatter API Contract Checker');
  console.log('=' .repeat(50));
  
  const frontendAPIs = flattenFrontendAPIs();
  const backendRoutes = flattenBackendRoutes();
  
  console.log(`\n📊 统计信息:`);
  console.log(`   前端API调用数: ${frontendAPIs.length}`);
  console.log(`   后端路由数: ${backendRoutes.length}`);
  
  let matches = 0;
  let mismatches = [];
  let missing_in_backend = [];
  let missing_in_frontend = [];
  
  // 检查前端API是否在后端存在
  console.log(`\n✅ 匹配的API:`);
  frontendAPIs.forEach(frontendAPI => {
    const normalizedFrontendPath = normalizePath(frontendAPI.path);
    const matched = backendRoutes.find(backendRoute => {
      const normalizedBackendPath = normalizePath(backendRoute.path);
      return normalizedBackendPath === normalizedFrontendPath && 
             backendRoute.method === frontendAPI.method;
    });
    
    if (matched) {
      matches++;
      console.log(`   ${frontendAPI.method} ${frontendAPI.path} ✓`);
    } else {
      missing_in_backend.push(frontendAPI);
    }
  });
  
  // 检查后端路由是否在前端被使用
  backendRoutes.forEach(backendRoute => {
    const normalizedBackendPath = normalizePath(backendRoute.path);
    const matched = frontendAPIs.find(frontendAPI => {
      const normalizedFrontendPath = normalizePath(frontendAPI.path);
      return normalizedFrontendPath === normalizedBackendPath && 
             frontendAPI.method === backendRoute.method;
    });
    
    if (!matched) {
      missing_in_frontend.push(backendRoute);
    }
  });
  
  // 报告不匹配的API
  if (missing_in_backend.length > 0) {
    console.log(`\n❌ 前端调用但后端缺失的API:`);
    missing_in_backend.forEach(api => {
      console.log(`   ${api.method} ${api.path} (${api.file})`);
    });
  }
  
  if (missing_in_frontend.length > 0) {
    console.log(`\n⚠️  后端定义但前端未使用的API:`);
    missing_in_frontend.forEach(route => {
      console.log(`   ${route.method} ${route.path}`);
    });
  }
  
  // 总结
  console.log(`\n📈 对应关系总结:`);
  console.log(`   匹配数: ${matches}`);
  console.log(`   前端缺失后端支持: ${missing_in_backend.length}`);
  console.log(`   后端未被前端使用: ${missing_in_frontend.length}`);
  
  const alignmentScore = (matches / (frontendAPIs.length + missing_in_frontend.length)) * 100;
  console.log(`   对应率: ${alignmentScore.toFixed(1)}%`);
  
  return {
    matches,
    missing_in_backend,
    missing_in_frontend,
    alignmentScore
  };
}

// 生成API契约文档
function generateAPIContract(results) {
  const contractDoc = `# Fechatter API 契约文档

## 📊 API 对应关系概览

- **匹配的API数量**: ${results.matches}
- **前端缺失后端支持**: ${results.missing_in_backend.length}
- **后端未被前端使用**: ${results.missing_in_frontend.length}
- **对应率**: ${results.alignmentScore.toFixed(1)}%

## 🔄 完整API映射表

### 认证相关 API
| 前端路径 | 后端路径 | 方法 | 状态 | 参数 |
|---------|---------|------|------|------|
| /signin | /signin | POST | ✅ | email, password |
| /signup | /signup | POST | ✅ | fullname, email, password, workspace |
| /logout | /logout | POST | ✅ | - |
| /logout_all | /logout_all | POST | ✅ | - |
| /refresh | /refresh | POST | ✅ | - |

### 聊天相关 API
| 前端路径 | 后端路径 | 方法 | 状态 | 参数 |
|---------|---------|------|------|------|
| /chat | /chat | GET | ✅ | - |
| /chat | /chat | POST | ✅ | name, chat_type, is_public, workspace_id |
| /chat/{id} | /chat/{id} | DELETE | ✅ | id |
| /chat/{id}/messages | /chat/{id}/messages | GET | ✅ | id, limit, offset |
| /chat/{id}/messages | /chat/{id}/messages | POST | ✅ | id, content, message_type |
| /chat/{id}/members | /chat/{id}/members | GET | ✅ | id |
| /chat/{id}/members | /chat/{id}/members | POST | ✅ | id, member_ids |
| /chat/{id}/members | /chat/{id}/members | DELETE | ✅ | id, member_ids |
| /chat/{id}/messages/search | /chat/{id}/messages/search | POST | ⚠️ | 路径不匹配 |

### 文件相关 API
| 前端路径 | 后端路径 | 方法 | 状态 | 参数 |
|---------|---------|------|------|------|
| /upload | /upload | POST | ✅ | file |
| /fix-files/{ws_id} | /fix-files/{ws_id} | POST | ✅ | ws_id |

### 工作空间相关 API
| 前端路径 | 后端路径 | 方法 | 状态 | 参数 |
|---------|---------|------|------|------|
| /workspaces | /workspaces | GET | ⚠️ | 路径不匹配 |
| /users | /users | GET | ✅ | - |
| /workspace/invite | - | POST | ❌ | 后端缺失 |
| /workspace/users/{userId} | - | DELETE | ❌ | 后端缺失 |

## 🚨 需要修复的问题

### 1. 前端调用但后端缺失
${results.missing_in_backend.map(api => `- ${api.method} ${api.path} (${api.file})`).join('\n')}

### 2. 路径不匹配问题
- **搜索API**: 前端 \`/chat/{id}/search\` vs 后端 \`/chat/{id}/messages/search\`
- **文件API**: 前端 \`/files/{path}\` vs 后端 \`/files/{ws_id}/{*path}\`
- **工作空间API**: 前端 \`/workspaces\` vs 后端 \`/workspaces\`

### 3. 后端已实现但前端未使用
${results.missing_in_frontend.map(route => `- ${route.method} ${route.path}`).join('\n')}

## 🛠️ 推荐修复方案

### 立即修复 (高优先级)
1. 统一搜索API路径
2. 修复工作空间API路径不匹配
3. 实现缺失的工作空间管理API

### 长期优化 (中优先级)
1. 建立API schema验证
2. 实现自动化契约测试
3. 添加API版本控制

---

*生成时间: ${new Date().toISOString()}*
`;

  fs.writeFileSync('API_CONTRACT.md', contractDoc);
  console.log('\n📄 API契约文档已生成: API_CONTRACT.md');
}

// 主函数
function main() {
  const results = checkAPIAlignment();
  generateAPIContract(results);
  
  // 如果有严重不匹配，退出码为1
  if (results.missing_in_backend.length > 0) {
    console.log('\n🚨 发现API不匹配问题，请检查并修复！');
    process.exit(1);
  } else {
    console.log('\n✅ API契约检查通过！');
  }
}

if (require.main === module) {
  main();
}

module.exports = {
  checkAPIAlignment,
  generateAPIContract,
  frontendAPIs,
  backendRoutes
}; 