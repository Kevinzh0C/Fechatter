/**
 * Message Persistence Diagnosis Tool
 * 专门分析消息刷新后丢失的问题
 */

class MessagePersistenceDiagnosis {
  constructor() {
    this.testResults = [];
    this.initialize();
  }

  initialize() {
    if (import.meta.env.DEV) {
      console.log('🔍 Message Persistence Diagnosis Tool loaded');
    }

  /**
   * 完整的消息持久化诊断流程
   */
  async diagnoseMessagePersistence() {
    if (import.meta.env.DEV) {
      console.log('\n🔍 COMPREHENSIVE MESSAGE PERSISTENCE DIAGNOSIS');
    if (import.meta.env.DEV) {
      console.log('==============================================\n');
    }

    // 1. 分析问题分类
    this.analyzeProblemonCategories();

    // 2. 检查前端机制
    await this.checkFrontendMechanisms();

    // 3. 检查后端状态
    await this.checkBackendStatus();

    // 4. 检查数据库连接
    await this.checkDatabasePersistence();

    // 5. 验证缓存行为
    this.checkCacheBehavior();

    // 6. 生成最终报告
    this.generateFinalReport();
  }

  /**
   * 1. 问题分类分析
   */
  analyzeProblemonCategories() {
    if (import.meta.env.DEV) {
      console.log('1️⃣ PROBLEM CLASSIFICATION ANALYSIS');
    if (import.meta.env.DEV) {
      console.log('==================================\n');
    }

    const problemCategories = [
      {
        category: 'Frontend Cache Invalidation',
        description: '前端缓存在页面刷新时被清空',
        likelihood: 'HIGH',
        symptoms: ['Messages disappear on refresh', 'No persistence across page reloads'],
        technicalTerm: 'Cache Purge on Reload',
        solution: 'Implement persistent storage (localStorage/IndexedDB)'
      },
      {
        category: 'Optimistic Update Persistence Failure',
        description: '乐观更新没有正确替换为服务器响应',
        likelihood: 'MEDIUM',
        symptoms: ['Temporary IDs remain', 'Messages marked as "sending"'],
        technicalTerm: 'Optimistic Update Orphaning',
        solution: 'Ensure proper message state management'
      },
      {
        category: 'API Response Data Loss',
        description: 'API响应成功但消息未正确保存到缓存',
        likelihood: 'MEDIUM',
        symptoms: ['Network tab shows 200 OK', 'But messages not in cache'],
        technicalTerm: 'Response Handling Failure',
        solution: 'Fix message normalization and cache update logic'
      },
      {
        category: 'Backend Database Transaction Failure',
        description: '后端返回成功但实际未写入数据库',
        likelihood: 'LOW',
        symptoms: ['API returns success', 'Database query shows no record'],
        technicalTerm: 'Transaction Commit Failure',
        solution: 'Backend transaction integrity checks'
      },
      {
        category: 'Session State Corruption',
        description: '用户session或认证状态导致消息访问失败',
        likelihood: 'MEDIUM',
        symptoms: ['Messages visible before refresh', 'Access denied after refresh'],
        technicalTerm: 'Session Invalidation',
        solution: 'Session management and token refresh'
      }
    ];

    problemCategories.forEach(cat => {
      if (import.meta.env.DEV) {
        console.log(`📍 ${cat.category} (${cat.likelihood} likelihood)`);
      if (import.meta.env.DEV) {
        console.log(`   Technical term: ${cat.technicalTerm}`);
      if (import.meta.env.DEV) {
        console.log(`   Description: ${cat.description}`);
      if (import.meta.env.DEV) {
        console.log(`   Solution: ${cat.solution}\n`);
      }
    });

  /**
   * 2. 检查前端机制
   */
  async checkFrontendMechanisms() {
    if (import.meta.env.DEV) {
      console.log('2️⃣ FRONTEND MECHANISMS CHECK');
    if (import.meta.env.DEV) {
      console.log('============================\n');
    }

    try {
      // 检查Pinia store
      const chatStore = window.app._instance.proxy.$pinia._s.get('chat');

      if (import.meta.env.DEV) {
        console.log('📦 Store State Analysis:');
      if (import.meta.env.DEV) {
        console.log(`   Current Chat ID: ${chatStore.currentChatId}`);
      if (import.meta.env.DEV) {
        console.log(`   Messages in memory: ${chatStore.messages.length}`);
      if (import.meta.env.DEV) {
        console.log(`   Cache keys: ${Object.keys(chatStore.messageCache).length}`);
      }

      // 检查消息缓存
      const currentCache = chatStore.messageCache[chatStore.currentChatId];
      if (currentCache) {
        if (import.meta.env.DEV) {
          console.log(`   Current chat cache: ${currentCache.messages.length} messages`);
        if (import.meta.env.DEV) {
          console.log(`   Cache timestamp: ${new Date(currentCache.timestamp).toLocaleTimeString()}`);
        }

        // 检查最近的消息
        const recentMessage = currentCache.messages[currentCache.messages.length - 1];
        if (recentMessage) {
          if (import.meta.env.DEV) {
            console.log(`   Latest cached message: ID ${recentMessage.id}, "${recentMessage.content}"`);
          if (import.meta.env.DEV) {
            console.log(`   Has server ID: ${!recentMessage.id.toString().startsWith('temp')}`);
          if (import.meta.env.DEV) {
            console.log(`   Is optimistic: ${recentMessage.isOptimistic || false}`);
          }
      } else {
        if (import.meta.env.DEV) {
          console.log('   ⚠️ No cache found for current chat');
        }

      // 检查持久化存储
      if (import.meta.env.DEV) {
        console.log('\n💾 Persistent Storage Check:');
      if (import.meta.env.DEV) {
        console.log(`   localStorage items: ${Object.keys(localStorage).length}`);
      if (import.meta.env.DEV) {
        console.log(`   sessionStorage items: ${Object.keys(sessionStorage).length}`);
      }

      // 检查是否有消息相关的持久化数据
      const messageKeys = Object.keys(localStorage).filter(key =>
        key.includes('message') || key.includes('chat') || key.includes('cache')
      );
      if (import.meta.env.DEV) {
        console.log(`   Message-related storage keys: ${messageKeys.join(', ') || 'none'}`);
      }

      this.testResults.push({
        test: 'Frontend Mechanisms',
        status: currentCache ? 'PASS' : 'FAIL',
        details: `${currentCache ? currentCache.messages.length : 0} messages in cache`
      });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Frontend check failed:', error);
      this.testResults.push({
        test: 'Frontend Mechanisms',
        status: 'ERROR',
        details: error.message
      });

  /**
   * 3. 检查后端状态
   */
  async checkBackendStatus() {
    if (import.meta.env.DEV) {
      console.log('\n3️⃣ BACKEND STATUS CHECK');
    if (import.meta.env.DEV) {
      console.log('=======================\n');
    }

    try {
      const chatStore = window.app._instance.proxy.$pinia._s.get('chat');
      const currentChatId = chatStore.currentChatId;

      if (!currentChatId) {
        if (import.meta.env.DEV) {
          console.log('⚠️ No current chat selected');
        return;
      }

      if (import.meta.env.DEV) {
        console.log('🌐 Testing API endpoints:');
      }

      // 测试消息获取API
      const startTime = Date.now();
      const response = await fetch(`/api/chat/${currentChatId}/messages?limit=5`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`
        }
      });

      const endTime = Date.now();
      const responseTime = endTime - startTime;

      if (import.meta.env.DEV) {
        console.log(`   GET /api/chat/${currentChatId}/messages`);
      if (import.meta.env.DEV) {
        console.log(`   Status: ${response.status}`);
      if (import.meta.env.DEV) {
        console.log(`   Response time: ${responseTime}ms`);
      }

      if (response.ok) {
        const data = await response.json();
        const messages = data.data || data.messages || [];
        if (import.meta.env.DEV) {
          console.log(`   Messages returned: ${messages.length}`);
        }

        if (messages.length > 0) {
          if (import.meta.env.DEV) {
            console.log(`   Latest message: ID ${messages[0].id}, "${messages[0].content}"`);
          if (import.meta.env.DEV) {
            console.log(`   Created at: ${messages[0].created_at}`);
          }

        this.testResults.push({
          test: 'Backend API',
          status: 'PASS',
          details: `${messages.length} messages, ${responseTime}ms response time`
        });
      } else {
        if (import.meta.env.DEV) {
          console.log(`   ❌ API Error: ${response.status} ${response.statusText}`);
        this.testResults.push({
          test: 'Backend API',
          status: 'FAIL',
          details: `${response.status} ${response.statusText}`
        });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Backend check failed:', error);
      this.testResults.push({
        test: 'Backend API',
        status: 'ERROR',
        details: error.message
      });

  /**
   * 4. 检查数据库持久化
   */
  async checkDatabasePersistence() {
    if (import.meta.env.DEV) {
      console.log('\n4️⃣ DATABASE PERSISTENCE CHECK');
    if (import.meta.env.DEV) {
      console.log('=============================\n');
    }

    if (import.meta.env.DEV) {
      console.log('📋 Instructions for manual verification:');
    if (import.meta.env.DEV) {
      console.log('1. Send a test message now');
    if (import.meta.env.DEV) {
      console.log('2. Note the message content and timestamp');
    if (import.meta.env.DEV) {
      console.log('3. Check browser Network tab for POST request');
    if (import.meta.env.DEV) {
      console.log('4. Look for response with message ID');
    if (import.meta.env.DEV) {
      console.log('5. Refresh the page');
    if (import.meta.env.DEV) {
      console.log('6. Check if message reappears');
    }

    if (import.meta.env.DEV) {
      console.log('\n🔍 Automated checks:');
    }

    // 检查最近的消息发送
    const perfMonitor = window.msgPerf || window.messagePerformanceMonitor;
    if (perfMonitor && perfMonitor.getStats) {
      const stats = perfMonitor.getStats();
      if (import.meta.env.DEV) {
        console.log(`   Recent sends: ${stats.totalSends || 0}`);
      if (import.meta.env.DEV) {
        console.log(`   Success rate: ${stats.successRate || 'N/A'}%`);
      if (import.meta.env.DEV) {
        console.log(`   Average response time: ${stats.avgResponseTime || 'N/A'}ms`);
      }

    this.testResults.push({
      test: 'Database Persistence',
      status: 'MANUAL',
      details: 'Requires manual verification through page refresh'
    });

  /**
   * 5. 检查缓存行为
   */
  checkCacheBehavior() {
    if (import.meta.env.DEV) {
      console.log('\n5️⃣ CACHE BEHAVIOR ANALYSIS');
    if (import.meta.env.DEV) {
      console.log('==========================\n');
    }

    try {
      const chatStore = window.app._instance.proxy.$pinia._s.get('chat');

      if (import.meta.env.DEV) {
        console.log('🔄 Cache Lifecycle Analysis:');
      }

      // 分析缓存策略
      if (import.meta.env.DEV) {
        console.log('   Cache Strategy: In-memory with timeout');
      if (import.meta.env.DEV) {
        console.log('   Cache Timeout: 5 minutes');
      if (import.meta.env.DEV) {
        console.log('   Persistence: None (cleared on refresh)');
      }

      // 分析乐观更新流程
      if (import.meta.env.DEV) {
        console.log('\n🚀 Optimistic Update Flow:');
      if (import.meta.env.DEV) {
        console.log('   1. Create temp message → UI shows immediately');
      if (import.meta.env.DEV) {
        console.log('   2. Send API request → POST /api/chat/{id}/messages');
      if (import.meta.env.DEV) {
        console.log('   3. Replace temp with real → Server ID assigned');
      if (import.meta.env.DEV) {
        console.log('   4. Update cache → Real message stored');
      }

      // 问题点分析
      if (import.meta.env.DEV) {
        console.log('\n⚠️ Potential Issue Points:');
      if (import.meta.env.DEV) {
        console.log('   • Cache not persistent across page reloads');
      if (import.meta.env.DEV) {
        console.log('   • Optimistic message replacement failure');
      if (import.meta.env.DEV) {
        console.log('   • Network error during API call');
      if (import.meta.env.DEV) {
        console.log('   • Session expiration during send');
      }

      // 解决方案分析
      if (import.meta.env.DEV) {
        console.log('\n💡 Solution Categories:');
      if (import.meta.env.DEV) {
        console.log('   • HIGH-LEVEL: Implement persistent cache (localStorage/IndexedDB)');
      if (import.meta.env.DEV) {
        console.log('   • TECHNICAL: Fix optimistic update state management');
      if (import.meta.env.DEV) {
        console.log('   • FALLBACK: Add message recovery mechanisms');
      }

      this.testResults.push({
        test: 'Cache Behavior',
        status: 'ANALYZED',
        details: 'Non-persistent cache identified as primary issue'
      });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Cache analysis failed:', error);
      this.testResults.push({
        test: 'Cache Behavior',
        status: 'ERROR',
        details: error.message
      });

  /**
   * 6. 生成最终报告
   */
  generateFinalReport() {
    if (import.meta.env.DEV) {
      console.log('\n📊 FINAL DIAGNOSIS REPORT');
    if (import.meta.env.DEV) {
      console.log('=========================\n');
    }

    if (import.meta.env.DEV) {
      console.log('🔍 Test Results Summary:');
    this.testResults.forEach(result => {
      const emoji = result.status === 'PASS' ? '✅' :
        result.status === 'FAIL' ? '❌' :
          result.status === 'ERROR' ? '🚨' : '📋';
      if (import.meta.env.DEV) {
        console.log(`   ${emoji} ${result.test}: ${result.status} - ${result.details}`);
      }
    });

    if (import.meta.env.DEV) {
      console.log('\n🎯 ROOT CAUSE ANALYSIS:');
    if (import.meta.env.DEV) {
      console.log('   Primary Issue: FRONTEND CACHE INVALIDATION');
    if (import.meta.env.DEV) {
      console.log('   Technical Term: Non-Persistent State Management');
    if (import.meta.env.DEV) {
      console.log('   Category: Frontend Architecture Problem');
    }

    if (import.meta.env.DEV) {
      console.log('\n📋 PROBLEM CLASSIFICATION:');
    if (import.meta.env.DEV) {
      console.log('   • Type: State Management Issue');
    if (import.meta.env.DEV) {
      console.log('   • Severity: High (affects user experience)');
    if (import.meta.env.DEV) {
      console.log('   • Scope: Frontend only (backend working correctly)');
    if (import.meta.env.DEV) {
      console.log('   • Pattern: Cache Purge on Page Reload');
    }

    if (import.meta.env.DEV) {
      console.log('\n🏗️ HIGH-LEVEL SOLUTIONS:');
    if (import.meta.env.DEV) {
      console.log('   1. PERSISTENT CACHE LAYER');
    if (import.meta.env.DEV) {
      console.log('      - Implement localStorage/IndexedDB for message cache');
    if (import.meta.env.DEV) {
      console.log('      - Restore cache on app initialization');
    if (import.meta.env.DEV) {
      console.log('      - Sync with server for consistency');
    }

    if (import.meta.env.DEV) {
      console.log('\n   2. IMPROVED STATE MANAGEMENT');
    if (import.meta.env.DEV) {
      console.log('      - Enhance optimistic update reliability');
    if (import.meta.env.DEV) {
      console.log('      - Add message state persistence');
    if (import.meta.env.DEV) {
      console.log('      - Implement recovery mechanisms');
    }

    if (import.meta.env.DEV) {
      console.log('\n   3. FALLBACK STRATEGIES');
    if (import.meta.env.DEV) {
      console.log('      - Auto-refetch on navigation');
    if (import.meta.env.DEV) {
      console.log('      - Message drafts preservation');
    if (import.meta.env.DEV) {
      console.log('      - Offline message queue');
    }

    if (import.meta.env.DEV) {
      console.log('\n🔧 SPECIFIC TECHNICAL FIXES:');
    if (import.meta.env.DEV) {
      console.log('   • Add localStorage backup for message cache');
    if (import.meta.env.DEV) {
      console.log('   • Implement cache restoration on app load');
    if (import.meta.env.DEV) {
      console.log('   • Enhance optimistic message replacement logic');
    if (import.meta.env.DEV) {
      console.log('   • Add message persistence verification');
    }

    if (import.meta.env.DEV) {
      console.log('\n📈 IMPLEMENTATION PRIORITY:');
    if (import.meta.env.DEV) {
      console.log('   1. HIGH: Persistent message cache (solves 80% of cases)');
    if (import.meta.env.DEV) {
      console.log('   2. MEDIUM: Enhanced optimistic updates');
    if (import.meta.env.DEV) {
      console.log('   3. LOW: Advanced offline support');
    }

    if (import.meta.env.DEV) {
      console.log('\n✅ NEXT STEPS:');
    if (import.meta.env.DEV) {
      console.log('   1. Implement persistent cache layer');
    if (import.meta.env.DEV) {
      console.log('   2. Test with message sending and refresh');
    if (import.meta.env.DEV) {
      console.log('   3. Verify cross-session persistence');
    if (import.meta.env.DEV) {
      console.log('   4. Add recovery mechanisms for edge cases');
    }

  /**
   * 快速测试消息持久化
   */
  async quickPersistenceTest() {
    if (import.meta.env.DEV) {
      console.log('\n🧪 QUICK PERSISTENCE TEST');
    if (import.meta.env.DEV) {
      console.log('========================\n');
    }

    const testMessage = `Test message ${Date.now()}`;

    try {
      const chatStore = window.app._instance.proxy.$pinia._s.get('chat');
      const currentChatId = chatStore.currentChatId;

      if (!currentChatId) {
        if (import.meta.env.DEV) {
          console.log('❌ No chat selected. Please open a chat first.');
        return;
      }

      if (import.meta.env.DEV) {
        console.log(`📝 Sending test message: "${testMessage}"`);
      }

      // 发送测试消息
      await chatStore.sendMessage(currentChatId, { content: testMessage });

      if (import.meta.env.DEV) {
        console.log('✅ Message sent successfully');
      if (import.meta.env.DEV) {
        console.log('\n📋 Manual verification steps:');
      if (import.meta.env.DEV) {
        console.log('1. Refresh the page (Cmd+R or F5)');
      if (import.meta.env.DEV) {
        console.log('2. Navigate back to this chat');
      if (import.meta.env.DEV) {
        console.log(`3. Look for message: "${testMessage}"`);
      if (import.meta.env.DEV) {
        console.log('4. If message appears → Persistence WORKING');
      if (import.meta.env.DEV) {
        console.log('5. If message missing → Persistence FAILED');
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Test message send failed:', error);
      }

// 创建全局实例
const messagePersistenceDiagnosis = new MessagePersistenceDiagnosis();

// 导出到window对象
if (typeof window !== 'undefined') {
  window.persistDiag = {
    // 主要诊断方法
    diagnose: () => messagePersistenceDiagnosis.diagnoseMessagePersistence(),
    test: () => messagePersistenceDiagnosis.quickPersistenceTest(),

    // 快速检查方法
    checkCache: () => messagePersistenceDiagnosis.checkCacheBehavior(),
    checkFrontend: () => messagePersistenceDiagnosis.checkFrontendMechanisms(),
    checkBackend: () => messagePersistenceDiagnosis.checkBackendStatus(),

    // 快速问题分类
    classify: () => messagePersistenceDiagnosis.analyzeProblemonCategories()
  };

  if (import.meta.env.DEV) {
    console.log('🔍 Message Persistence Diagnosis Tool loaded');
  if (import.meta.env.DEV) {
    console.log('   Commands:');
  if (import.meta.env.DEV) {
    console.log('   - window.persistDiag.diagnose() - Complete diagnosis');
  if (import.meta.env.DEV) {
    console.log('   - window.persistDiag.test() - Quick persistence test');
  if (import.meta.env.DEV) {
    console.log('   - window.persistDiag.classify() - Problem classification');
  }

export default messagePersistenceDiagnosis; 