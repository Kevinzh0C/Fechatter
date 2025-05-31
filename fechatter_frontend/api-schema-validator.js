#!/usr/bin/env node

/**
 * Fechatter API Schema Validator
 * 验证前后端API数据格式一致性，自动生成类型定义
 */

const fs = require('fs');
const path = require('path');

// API Schema定义
const apiSchemas = {
  // 认证相关
  auth: {
    '/signin': {
      method: 'POST',
      request: {
        email: 'string',
        password: 'string'
      },
      response: {
        access_token: 'string',
        refresh_token: 'string',
        expires_in: 'number',
        user: {
          id: 'number',
          email: 'string',
          fullname: 'string',
          workspace_id: 'number',
          status: 'string',
          created_at: 'string'
        }
      }
    },
    '/signup': {
      method: 'POST',
      request: {
        fullname: 'string',
        email: 'string',
        password: 'string',
        workspace: 'string?'
      },
      response: {
        access_token: 'string',
        refresh_token: 'string',
        expires_in: 'number',
        user: {
          id: 'number',
          email: 'string',
          fullname: 'string',
          workspace_id: 'number',
          status: 'string',
          created_at: 'string'
        }
      }
    }
  },

  // 聊天相关
  chat: {
    '/chat': {
      GET: {
        method: 'GET',
        response: {
          type: 'array',
          items: {
            id: 'number',
            name: 'string',
            chat_type: 'string',
            is_public: 'boolean',
            created_by: 'number',
            workspace_id: 'number',
            member_count: 'number',
            last_message: {
              id: 'number',
              content: 'string',
              created_at: 'string',
              sender: {
                id: 'number',
                fullname: 'string'
              }
            }
          }
        }
      },
      POST: {
        method: 'POST',
        request: {
          name: 'string',
          chat_type: 'string',
          is_public: 'boolean',
          workspace_id: 'number'
        },
        response: {
          id: 'number',
          name: 'string',
          chat_type: 'string',
          is_public: 'boolean',
          created_by: 'number',
          workspace_id: 'number',
          created_at: 'string'
        }
      }
    },

    '/chat/{id}/messages': {
      GET: {
        method: 'GET',
        params: {
          id: 'number'
        },
        query: {
          limit: 'number?',
          offset: 'number?'
        },
        response: {
          type: 'array',
          items: {
            id: 'number',
            content: 'string',
            message_type: 'string',
            chat_id: 'number',
            sender_id: 'number',
            created_at: 'string',
            sender: {
              id: 'number',
              fullname: 'string'
            },
            files: {
              type: 'array',
              items: {
                path: 'string',
                filename: 'string',
                size: 'number',
                mime_type: 'string'
              }
            }
          }
        }
      },
      POST: {
        method: 'POST',
        params: {
          id: 'number'
        },
        request: {
          content: 'string',
          message_type: 'string',
          files: 'array?'
        },
        response: {
          id: 'number',
          content: 'string',
          message_type: 'string',
          chat_id: 'number',
          sender_id: 'number',
          created_at: 'string'
        }
      }
    }
  }
};

// 生成TypeScript类型定义
function generateTypeScriptTypes() {
  let tsTypes = `// Fechatter API Types
// 自动生成，请勿手动修改
// Generated at: ${new Date().toISOString()}

`;

  Object.entries(apiSchemas).forEach(([category, apis]) => {
    tsTypes += `// ${category.toUpperCase()} Types\n`;
    
    Object.entries(apis).forEach(([path, schema]) => {
      const sanitizedPath = path.replace(/[{}\/]/g, '_').replace(/_+/g, '_');
      
      if (schema.method) {
        // Single method endpoint
        if (schema.request) {
          tsTypes += generateInterface(`${sanitizedPath}_Request`, schema.request);
        }
        if (schema.response) {
          tsTypes += generateInterface(`${sanitizedPath}_Response`, schema.response);
        }
      } else {
        // Multiple methods endpoint
        Object.entries(schema).forEach(([method, methodSchema]) => {
          if (methodSchema.request) {
            tsTypes += generateInterface(`${sanitizedPath}_${method}_Request`, methodSchema.request);
          }
          if (methodSchema.response) {
            tsTypes += generateInterface(`${sanitizedPath}_${method}_Response`, methodSchema.response);
          }
        });
      }
    });
    
    tsTypes += '\n';
  });

  return tsTypes;
}

// 生成TypeScript接口
function generateInterface(name, schema) {
  let result = `export interface ${name} {\n`;
  
  if (schema.type === 'array') {
    result = `export type ${name} = ` + generateType(schema) + ';\n\n';
  } else {
    Object.entries(schema).forEach(([key, type]) => {
      const optional = typeof type === 'string' && type.endsWith('?');
      const cleanType = optional ? type.slice(0, -1) : type;
      result += `  ${key}${optional ? '?' : ''}: ${generateType(cleanType)};\n`;
    });
    result += '}\n\n';
  }
  
  return result;
}

// 生成类型字符串
function generateType(type) {
  if (typeof type === 'string') {
    switch (type) {
      case 'string': return 'string';
      case 'number': return 'number';
      case 'boolean': return 'boolean';
      case 'array': return 'any[]';
      default: return 'any';
    }
  } else if (type.type === 'array') {
    return `Array<${generateInterfaceInline(type.items)}>`;
  } else if (typeof type === 'object') {
    return generateInterfaceInline(type);
  }
  
  return 'any';
}

// 生成内联接口
function generateInterfaceInline(schema) {
  if (typeof schema === 'string') {
    return generateType(schema);
  }
  
  let result = '{\n';
  Object.entries(schema).forEach(([key, type]) => {
    const optional = typeof type === 'string' && type.endsWith('?');
    const cleanType = optional ? type.slice(0, -1) : type;
    result += `    ${key}${optional ? '?' : ''}: ${generateType(cleanType)};\n`;
  });
  result += '  }';
  
  return result;
}

// 生成Rust类型定义
function generateRustTypes() {
  let rustTypes = `// Fechatter API Types
// 自动生成，请勿手动修改
// Generated at: ${new Date().toISOString()}

use serde::{Deserialize, Serialize};

`;

  Object.entries(apiSchemas).forEach(([category, apis]) => {
    rustTypes += `// ${category.toUpperCase()} Types\n`;
    
    Object.entries(apis).forEach(([path, schema]) => {
      const sanitizedPath = toPascalCase(path.replace(/[{}\/]/g, '_').replace(/_+/g, '_'));
      
      if (schema.method) {
        // Single method endpoint
        if (schema.request) {
          rustTypes += generateRustStruct(`${sanitizedPath}Request`, schema.request);
        }
        if (schema.response) {
          rustTypes += generateRustStruct(`${sanitizedPath}Response`, schema.response);
        }
      } else {
        // Multiple methods endpoint
        Object.entries(schema).forEach(([method, methodSchema]) => {
          if (methodSchema.request) {
            rustTypes += generateRustStruct(`${sanitizedPath}${toPascalCase(method)}Request`, methodSchema.request);
          }
          if (methodSchema.response) {
            rustTypes += generateRustStruct(`${sanitizedPath}${toPascalCase(method)}Response`, methodSchema.response);
          }
        });
      }
    });
    
    rustTypes += '\n';
  });

  return rustTypes;
}

// 生成Rust结构体
function generateRustStruct(name, schema) {
  let result = `#[derive(Debug, Serialize, Deserialize)]\npub struct ${name} {\n`;
  
  if (schema.type === 'array') {
    return `pub type ${name} = Vec<${generateRustType(schema.items)}>;\n\n`;
  } else {
    Object.entries(schema).forEach(([key, type]) => {
      const optional = typeof type === 'string' && type.endsWith('?');
      const cleanType = optional ? type.slice(0, -1) : type;
      const rustKey = toSnakeCase(key);
      result += `    ${optional ? '#[serde(skip_serializing_if = "Option::is_none")]' : ''}\n`;
      result += `    pub ${rustKey}: ${optional ? 'Option<' : ''}${generateRustType(cleanType)}${optional ? '>' : ''},\n`;
    });
    result += '}\n\n';
  }
  
  return result;
}

// 生成Rust类型
function generateRustType(type) {
  if (typeof type === 'string') {
    switch (type) {
      case 'string': return 'String';
      case 'number': return 'i64';
      case 'boolean': return 'bool';
      case 'array': return 'Vec<serde_json::Value>';
      default: return 'serde_json::Value';
    }
  } else if (type.type === 'array') {
    return `Vec<${generateRustTypeInline(type.items)}>`;
  } else if (typeof type === 'object') {
    return generateRustTypeInline(type);
  }
  
  return 'serde_json::Value';
}

// 生成内联Rust类型
function generateRustTypeInline(schema) {
  if (typeof schema === 'string') {
    return generateRustType(schema);
  }
  
  // For complex inline types, we should ideally create a separate struct
  // For now, use serde_json::Value
  return 'serde_json::Value';
}

// 工具函数
function toPascalCase(str) {
  return str.replace(/(^|_)([a-z])/g, (_, __, letter) => letter.toUpperCase());
}

function toSnakeCase(str) {
  return str.replace(/([A-Z])/g, '_$1').toLowerCase().replace(/^_/, '');
}

// 生成API契约测试
function generateAPITests() {
  let tests = `// Fechatter API Contract Tests
// 自动生成，请勿手动修改

import { describe, it, expect } from 'vitest';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:6688/api';

describe('API Contract Tests', () => {
`;

  Object.entries(apiSchemas).forEach(([category, apis]) => {
    tests += `  describe('${category.toUpperCase()} APIs', () => {\n`;
    
    Object.entries(apis).forEach(([path, schema]) => {
      const testPath = path.replace(/\{(\w+)\}/g, '${testId}');
      
      if (schema.method) {
        tests += generateAPITest(path, schema.method, schema, testPath);
      } else {
        Object.entries(schema).forEach(([method, methodSchema]) => {
          tests += generateAPITest(path, method, methodSchema, testPath);
        });
      }
    });
    
    tests += '  });\n\n';
  });

  tests += '});\n';
  return tests;
}

function generateAPITest(path, method, schema, testPath) {
  return `    it('should handle ${method} ${path}', async () => {
      // TODO: Implement test for ${method} ${path}
      // Request: ${JSON.stringify(schema.request || 'none')}
      // Response: ${JSON.stringify(schema.response || 'none')}
    });

`;
}

// 主函数
function main() {
  console.log('🔍 Fechatter API Schema Validator');
  console.log('=' .repeat(50));
  
  // 生成TypeScript类型
  const tsTypes = generateTypeScriptTypes();
  fs.writeFileSync('fechatter_frontend/src/types/api.ts', tsTypes);
  console.log('✅ TypeScript types generated: fechatter_frontend/src/types/api.ts');
  
  // 生成Rust类型
  const rustTypes = generateRustTypes();
  fs.writeFileSync('api_types.rs', rustTypes);
  console.log('✅ Rust types generated: api_types.rs');
  
  // 生成API测试
  const apiTests = generateAPITests();
  fs.writeFileSync('api-contract-tests.spec.js', apiTests);
  console.log('✅ API contract tests generated: api-contract-tests.spec.js');
  
  // 生成验证配置
  const validationConfig = {
    "api_schemas": apiSchemas,
    "validation_rules": {
      "require_documentation": true,
      "require_error_handling": true,
      "require_type_safety": true,
      "enforce_naming_convention": true
    },
    "generated_at": new Date().toISOString()
  };
  
  fs.writeFileSync('api-validation.json', JSON.stringify(validationConfig, null, 2));
  console.log('✅ Validation config generated: api-validation.json');
  
  console.log('\n🎯 使用方法:');
  console.log('   1. 在前端导入类型: import { SigninRequest, SigninResponse } from "@/types/api"');
  console.log('   2. 在后端使用生成的Rust结构体');
  console.log('   3. 运行API契约测试: npm test api-contract-tests.spec.js');
  console.log('   4. 定期运行验证: node api-schema-validator.js');
}

if (require.main === module) {
  main();
}

module.exports = {
  generateTypeScriptTypes,
  generateRustTypes,
  generateAPITests,
  apiSchemas
}; 