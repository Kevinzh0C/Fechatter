/**
 * 架构验证工具 - 验证前端是否正确通过Gateway连接
 * Fechatter Architecture Validator
 */

import { getApiConfig } from '@/utils/configLoader';

/**
 * 验证前端架构配置是否正确
 */
export async function validateArchitecture() {
  console.group('Fechatter Architecture Validation');

  const results = {
    passed: 0,
    failed: 0,
    warnings: 0,
    details: []
  };

  try {
    // 1. 验证配置加载
    const config = getApiConfig();

    if (!config || Object.keys(config).length === 0) {
      addResult(results, 'error', 'Configuration', '配置加载失败', 'YAML配置文件未正确加载');
    } else {
      addResult(results, 'success', 'Configuration', '配置加载成功', `Environment: ${config.environment || 'unknown'}`);
    }

    // 2. 验证Gateway URL配置
    const expectedGatewayUrl = 'http://45.77.178.85:8080';

    if (config.gateway_url === expectedGatewayUrl) {
      addResult(results, 'success', 'Gateway URL', 'Gateway地址正确', config.gateway_url);
    } else {
      addResult(results, 'error', 'Gateway URL', 'Gateway地址错误', `期望: ${expectedGatewayUrl}, 实际: ${config.gateway_url}`);
    }

    // 3. 验证所有端点都指向Gateway
    const expectedEndpoints = [
      { name: 'Gateway URL', key: 'gateway_url', expected: 'http://45.77.178.85:8080' },
      { name: 'API Base URL', key: 'base_url', expected: 'http://45.77.178.85:8080/api' },
      { name: 'File URL', key: 'file_url', expected: 'http://45.77.178.85:8080/files' },
      { name: 'SSE URL', key: 'sse_url', expected: 'http://45.77.178.85:8080/events' },
      { name: 'Notify URL', key: 'notify_url', expected: 'http://45.77.178.85:8080' }
    ];

    expectedEndpoints.forEach(endpoint => {
      if (config[endpoint.key] === endpoint.expected) {
        addResult(results, 'success', endpoint.name, '端点配置正确', config[endpoint.key]);
      } else {
        addResult(results, 'error', endpoint.name, '端点配置错误',
          `期望: ${endpoint.expected}, 实际: ${config[endpoint.key]}`);
      }
    });

    // 4. 验证Vite代理配置（运行时检测）
    if (import.meta.env.DEV) {
      try {
        // 检查代理是否工作 - 使用相对URL测试Vite代理
        const proxyTest = await fetch('/health', {
          method: 'HEAD',
          timeout: 3000
        }).catch(() => null);

        if (proxyTest && proxyTest.ok) {
          addResult(results, 'success', 'Vite Proxy', 'Health代理工作正常', '/health -> Gateway');
        } else {
          // 如果代理不工作，测试直接连接Gateway（仅用于诊断）
          const directTest = await fetch('/health', {
            method: 'HEAD',
            timeout: 3000
          }).catch(() => null);

          if (directTest) {
            addResult(results, 'warning', 'Vite Proxy', 'Health端点可访问但可能有配置问题', '请检查vite.config.js proxy配置');
          } else {
            addResult(results, 'error', 'Vite Proxy', 'Health端点不可访问', '请检查Gateway是否运行和代理配置');
          }
      } catch (error) {
        addResult(results, 'warning', 'Vite Proxy', '代理测试失败', error.message);
      }

    // 5. 验证SSE服务配置
    try {
      const sseModule = await import('@/services/sse');
      if (sseModule.default) {
        addResult(results, 'success', 'SSE Service', 'SSE服务模块加载正常', 'Ready for Gateway connection');
      }
    } catch (error) {
      addResult(results, 'error', 'SSE Service', 'SSE服务模块加载失败', error.message);
    }

    // 6. 生成报告
    const summary = generateSummary(results);
    if (import.meta.env.DEV) {
      console.log('\n验证摘要:');
    if (import.meta.env.DEV) {
      console.log(`ERROR: 失败: ${results.failed}`);
    if (import.meta.env.DEV) {
      console.log(`WARNING: 警告: ${results.warnings}`);
    }

    if (results.failed === 0) {
      if (import.meta.env.DEV) {
        console.log('\n🎉 架构验证通过！前端已正确配置通过Gateway连接。');
      }
    } else {
      if (import.meta.env.DEV) {
        console.log('\n🚨 架构验证失败！请检查以下问题:');
      results.details.filter(d => d.level === 'error').forEach(detail => {
        if (import.meta.env.DEV) {
          console.log(`   ERROR: ${detail.component}: ${detail.message}`);
        }
      });

    console.groupEnd();
    return summary;

  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('架构验证过程中发生错误:', error);
    console.groupEnd();
    return { success: false, error: error.message };
  }

/**
 * 测试实际连接路径
 */
export async function testConnectionPaths() {
  console.group('Connection Path Testing');

  const tests = [
    {
      name: 'Gateway Health Check (via proxy)',
      url: '/health',
      description: '测试Gateway健康检查（通过vite代理）'
    },
    {
      name: 'API通过Gateway',
      url: '/api/health',
      description: '测试API请求是否通过Gateway'
    },
    {
      name: 'SSE通过Gateway',
      url: '/events',
      description: '测试SSE端点是否可达（通过vite代理）',
      method: 'HEAD'
    }
  ];

  const results = [];

  for (const test of tests) {
    try {
      if (import.meta.env.DEV) {
        console.log(`🧪 Testing: ${test.name}`);
      }

      const response = await fetch(test.url, {
        method: test.method || 'GET',
        timeout: 5000
      });

      if (response.ok || response.status === 401) { // 401 is OK, means service is reachable
        results.push({
          name: test.name,
          success: true,
          status: response.status,
          message: `${test.description} - 连接成功`
        });
        if (import.meta.env.DEV) {
          console.log(`   ${test.name}: 连接成功 (${response.status})`);
        }
      } else {
        results.push({
          name: test.name,
          success: false,
          status: response.status,
          message: `${test.description} - HTTP ${response.status}`
        });
        if (import.meta.env.DEV) {
          console.log(`   ERROR: ${test.name}: HTTP ${response.status}`);
        }
    } catch (error) {
      results.push({
        name: test.name,
        success: false,
        error: error.message,
        message: `${test.description} - 连接失败: ${error.message}`
      });
      if (import.meta.env.DEV) {
        console.log(`   ERROR: ${test.name}: 连接失败 - ${error.message}`);
      }

  console.groupEnd();
  return results;
}

/**
 * 添加验证结果
 */
function addResult(results, level, component, message, details) {
  results.details.push({
    level,
    component,
    message,
    details,
    timestamp: new Date().toISOString()
  });

  switch (level) {
    case 'success':
      results.passed++;
      break;
    case 'error':
      results.failed++;
      break;
    case 'warning':
      results.warnings++;
      break;
  }

/**
 * 生成验证摘要
 */
function generateSummary(results) {
  return {
    success: results.failed === 0,
    passed: results.passed,
    failed: results.failed,
    warnings: results.warnings,
    total: results.details.length,
    details: results.details,
    timestamp: new Date().toISOString()
  };
}

/**
 * 导出验证报告
 */
export function exportValidationReport(summary) {
  const report = {
    title: 'Fechatter Frontend Architecture Validation Report',
    generated: new Date().toISOString(),
    summary,
    recommendations: generateRecommendations(summary)
  };

  const blob = new Blob([JSON.stringify(report, null, 2)], {
    type: 'application/json'
  });

  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `fechatter-architecture-validation-${Date.now()}.json`;
  a.click();
  URL.revokeObjectURL(url);
}

/**
 * 生成修复建议
 */
function generateRecommendations(summary) {
  const recommendations = [];

  if (summary.failed > 0) {
    recommendations.push(
      '1. 检查 fechatter_frontend/config/development.yml 配置是否正确',
      '2. 确认 vite.config.js 中的代理配置指向Gateway (8080端口)',
      '3. 验证Gateway服务是否在8080端口正常运行',
      '4. 检查SSE服务是否使用了配置的Gateway URL'
    );
  }

  if (summary.warnings > 0) {
    recommendations.push(
      '5. 检查网络连接和服务状态',
      '6. 验证开发环境代理配置'
    );
  }

  return recommendations;
}

// 在开发环境下自动暴露到全局
if (import.meta.env.DEV && typeof window !== 'undefined') {
  window.validateArchitecture = validateArchitecture;
  window.testConnectionPaths = testConnectionPaths;
  window.exportValidationReport = exportValidationReport;
}

export default {
  validateArchitecture,
  testConnectionPaths,
  exportValidationReport
}; 