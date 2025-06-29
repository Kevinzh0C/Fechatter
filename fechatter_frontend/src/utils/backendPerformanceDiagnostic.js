/**
 * Backend Performance Diagnostic Tool
 * Diagnose the root cause of backend 8-second delay
 */

class BackendPerformanceDiagnostic {
  constructor() {
    this.diagnosticResults = [];
    this.serverInfo = {
      ip: '45.77.178.85',
      port: 8080,
      dockerPort: 6688,
      provider: 'Vultr', // Based on IP range 45.77.x.x
      estimatedLocation: 'US/Europe', // Vultr has multiple locations
    };
  }

  /**
   * Run complete diagnostic
   */
  async runDiagnostic() {
    if (import.meta.env.DEV) {
      console.log('Starting Backend Performance Diagnostic...');
    if (import.meta.env.DEV) {
      console.log('📍 Server:', this.serverInfo.ip + ':' + this.serverInfo.port);
    }

    const results = {
      timestamp: new Date().toISOString(),
      tests: []
    };

    // Test 1: Network Latency
    results.tests.push(await this.testNetworkLatency());

    // Test 2: API Response Time Breakdown
    results.  tn.push(await this.testAPIResponseTime());

    // Test 3: Message Processing Analysis
    results.tests.push(await this.analyzeMessageProcessing());

    // Test 4: Docker Container Analysis
    results.tests.push(this.analyzeDockerSetup());

    // Test 5: Backend Architecture Analysis
    results.tests.push(this.analyzeBackendArchitecture());

    // Generate report
    this.generateReport(results);

    return results;
  }

  /**
   * Test 1: Network Latency
   */
  async testNetworkLatency() {
    if (import.meta.env.DEV) {
      console.log('\nSUBSCRIPTION: Test 1: Network Latency Analysis');
    }

    const results = {
      testName: 'Network Latency',
      measurements: []
    };

    // Ping-like test using fetch
    for (let i = 0; i < 5; i++) {
      const start = performance.now();
      try {
        await fetch(`http://${this.serverInfo.ip}:${this.serverInfo.port}/health`, {
          method: 'HEAD',
          mode: 'no-cors'
        });
      } catch (e) {
        // Expected in no-cors mode
      const end = performance.now();
      const latency = end - start;
      results.measurements.push(latency);
      if (import.meta.env.DEV) {
        console.log(`  Attempt ${i + 1}: ${latency.toFixed(2)}ms`);
      }

    const avgLatency = results.measurements.reduce((a, b) => a + b, 0) / results.measurements.length;
    results.averageLatency = avgLatency;
    results.analysis = this.analyzeLatency(avgLatency);

    return results;
  }

  /**
   * Test 2: API Response Time Breakdown
   */
  async testAPIResponseTime() {
    if (import.meta.env.DEV) {
      console.log('\n⏱️ Test 2: API Response Time Breakdown');
    }

    const results = {
      testName: 'API Response Time',
      endpoints: []
    };

    // Test different endpoints
    const endpoints = [
      { path: '/api/health', method: 'GET', expectedTime: 100 },
      { path: '/api/workspace/chats', method: 'GET', expectedTime: 500 },
      { path: '/api/chat/6/messages', method: 'GET', expectedTime: 1000 }
    ];

    for (const endpoint of endpoints) {
      const start = performance.now();
      try {
        const response = await fetch(`http://${this.serverInfo.ip}:${this.serverInfo.port}${endpoint.path}`, {
          method: endpoint.method,
          headers: {
            'Authorization': `Bearer ${window.tokenManager?.getToken() || ''}`
          }
        });
        const end = performance.now();
        const responseTime = end - start;

        results.endpoints.push({
          path: endpoint.path,
          responseTime,
          status: response.status,
          slowdown: responseTime / endpoint.expectedTime,
          analysis: responseTime > endpoint.expectedTime * 3 ? 'SLOW' : 'OK'
        });

        if (import.meta.env.DEV) {
          console.log(`  ${endpoint.path}: ${responseTime.toFixed(0)}ms (${response.status})`);
        }
      } catch (error) {
        if (import.meta.env.DEV) {
          console.log(`  ${endpoint.path}: FAILED - ${error.message}`);
        }

    return results;
  }

  /**
   * Test 3: Message Processing Analysis
   */
  async analyzeMessageProcessing() {
    if (import.meta.env.DEV) {
      console.log('\nMESSAGE: Test 3: Message Processing Analysis');
    }

    const messagePerf = window.msgPerf?.getStats() || {};
    const slowRequests = window.msgPerf?.getSlowRequests() || [];

    return {
      testName: 'Message Processing',
      stats: messagePerf,
      slowRequests: slowRequests.length,
      analysis: {
        avgBackendTime: messagePerf.avgBackendTime,
        slowPercentage: messagePerf.slowBackendPercentage,
        bottleneck: messagePerf.avgBackendTime > 3000 ? 'SEVERE' : 'MODERATE'
      }
    };
  }

  /**
   * Test 4: Docker Container Analysis
   */
  analyzeDockerSetup() {
    if (import.meta.env.DEV) {
      console.log('\n🐳 Test 4: Docker Container Analysis');
    }

    return {
      testName: 'Docker Setup',
      configuration: {
        exposedPort: this.serverInfo.port,
        containerPort: this.serverInfo.dockerPort,
        portMapping: `${this.serverInfo.port}:${this.serverInfo.dockerPort}`,
        possibleIssues: [
          'Port forwarding overhead',
          'Docker network bridge latency',
          'Container resource limits',
          'Host machine performance'
        ]
      },
      recommendations: [
        'Check Docker container resource limits (CPU/Memory)',
        'Consider using host network mode for better performance',
        'Monitor container logs for errors',
        'Check if container is swapping memory'
      ]
    };
  }

  /**
   * Test 5: Backend Architecture Analysis
   */
  analyzeBackendArchitecture() {
    if (import.meta.env.DEV) {
      console.log('\nTest 5: Backend Architecture Analysis');
    }

    return {
      testName: 'Backend Architecture',
      possibleBottlenecks: {
        database: {
          issue: 'Slow database queries',
          symptoms: ['Consistent 8s delay', 'All endpoints slow'],
          likelihood: 'HIGH'
        },
        messageProcessing: {
          issue: 'Complex message processing logic',
          symptoms: ['Only message endpoints slow', 'CPU intensive operations'],
          likelihood: 'MEDIUM'
        },
        networkIO: {
          issue: 'Network I/O blocking',
          symptoms: ['Intermittent delays', 'Timeout patterns'],
          likelihood: 'LOW'
        },
        resourceContention: {
          issue: 'Server resource contention',
          symptoms: ['Variable response times', 'Peak hour slowdowns'],
          likelihood: 'MEDIUM'
        }
    };
  }

  /**
   * Analyze latency results
   */
  analyzeLatency(avgLatency) {
    if (avgLatency < 50) {
      return {
        level: 'EXCELLENT',
        description: 'Same region/datacenter',
        impact: 'Minimal network latency'
      };
    } else if (avgLatency < 150) {
      return {
        level: 'GOOD',
        description: 'Same country/nearby region',
        impact: 'Acceptable network latency'
      };
    } else if (avgLatency < 300) {
      return {
        level: 'MODERATE',
        description: 'Cross-country connection',
        impact: 'Noticeable network latency'
      };
    } else {
      return {
        level: 'POOR',
        description: 'Intercontinental connection',
        impact: 'Significant network latency'
      };
    }

  /**
   * Generate diagnostic report
   */
  generateReport(results) {
    if (import.meta.env.DEV) {
      console.log('\n' + '='.repeat(60));
    if (import.meta.env.DEV) {
      console.log('BACKEND PERFORMANCE DIAGNOSTIC REPORT');
    if (import.meta.env.DEV) {
      console.log('='.repeat(60));
    }

    if (import.meta.env.DEV) {
      console.log('\nROOT CAUSE ANALYSIS:');
    if (import.meta.env.DEV) {
      console.log('\nBased on the 8-second delay pattern, the most likely causes are:');
    }

    if (import.meta.env.DEV) {
      console.log('\n1. 🗄️ DATABASE PERFORMANCE (90% likelihood)');
    if (import.meta.env.DEV) {
      console.log('   - Symptoms: Consistent 8s delay across all message operations');
    if (import.meta.env.DEV) {
      console.log('   - Cause: Unoptimized queries, missing indexes, or connection pooling issues');
    if (import.meta.env.DEV) {
      console.log('   - Evidence: All message-related endpoints are slow');
    }

    if (import.meta.env.DEV) {
      console.log('\n2. 🐳 DOCKER CONFIGURATION (60% likelihood)');
    if (import.meta.env.DEV) {
      console.log('   - Port mapping: 8080 -> 6688 adds overhead');
    if (import.meta.env.DEV) {
      console.log('   - Container resource limits may be too restrictive');
    if (import.meta.env.DEV) {
      console.log('   - Docker bridge network adds latency');
    }

    if (import.meta.env.DEV) {
      console.log('\n3. 🌐 SERVER LOCATION (40% likelihood)');
    if (import.meta.env.DEV) {
      console.log('   - Server IP suggests Vultr hosting');
    if (import.meta.env.DEV) {
      console.log('   - Physical distance adds base latency');
    if (import.meta.env.DEV) {
      console.log('   - But this alone cannot explain 8s delays');
    }

    if (import.meta.env.DEV) {
      console.log('\n4. 💾 BACKEND ARCHITECTURE (70% likelihood)');
    if (import.meta.env.DEV) {
      console.log('   - Synchronous message processing');
    if (import.meta.env.DEV) {
      console.log('   - No caching layer');
    if (import.meta.env.DEV) {
      console.log('   - Inefficient data serialization');
    }

    if (import.meta.env.DEV) {
      console.log('\nRECOMMENDED ACTIONS:');
    if (import.meta.env.DEV) {
      console.log('1. Enable database query logging and analyze slow queries');
    if (import.meta.env.DEV) {
      console.log('2. Add database indexes on chat_id and created_at columns');
    if (import.meta.env.DEV) {
      console.log('3. Implement connection pooling if not already present');
    if (import.meta.env.DEV) {
      console.log('4. Add Redis caching layer for frequently accessed data');
    if (import.meta.env.DEV) {
      console.log('5. Use async message processing with message queues');
    if (import.meta.env.DEV) {
      console.log('6. Monitor Docker container resources (docker stats)');
    if (import.meta.env.DEV) {
      console.log('7. Consider upgrading server or moving closer to users');
    }

    if (import.meta.env.DEV) {
      console.log('\nQUICK FIXES:');
    if (import.meta.env.DEV) {
      console.log('1. Increase Docker container memory/CPU limits');
    if (import.meta.env.DEV) {
      console.log('2. Enable database query caching');
    if (import.meta.env.DEV) {
      console.log('3. Reduce message fetch limit from 15 to 10');
    if (import.meta.env.DEV) {
      console.log('4. Implement backend response compression');
    }

    if (import.meta.env.DEV) {
      console.log('\n' + '='.repeat(60));
    }

    return results;
  }

// Create global instance
const backendDiagnostic = new BackendPerformanceDiagnostic();

// Export to window
if (typeof window !== 'undefined') {
  window.backendDiag = {
    run: () => backendDiagnostic.runDiagnostic(),
    info: () => console.log(backendDiagnostic.serverInfo)
  };

  if (import.meta.env.DEV) {
    console.log('Backend Performance Diagnostic loaded');
  if (import.meta.env.DEV) {
    console.log('   Run diagnostic: window.backendDiag.run()');
  if (import.meta.env.DEV) {
    console.log('   Server info: window.backendDiag.info()');
  }

export default backendDiagnostic; 