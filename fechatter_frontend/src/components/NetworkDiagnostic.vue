<template>
  <div class="network-diagnostic p-4">
    <h2 class="text-xl font-bold mb-4">Network Diagnostic Tool</h2>
    
    <button 
      @click="runDiagnostics" 
      :disabled="running"
      class="px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600 disabled:bg-gray-400"
    >
      {{ running ? 'Running...' : 'Run Network Diagnostics' }}
    </button>
    
    <div v-if="results" class="mt-6 space-y-4">
      <!-- Environment Info -->
      <div class="bg-gray-100 p-4 rounded">
        <h3 class="font-bold mb-2">Environment</h3>
        <div class="text-sm space-y-1">
          <div><strong>Browser:</strong> {{ results.environment.userAgent.split(' ').slice(-2).join(' ') }}</div>
          <div><strong>Online:</strong> {{ results.environment.onLine ? 'Yes' : 'No' }}</div>
          <div><strong>Current URL:</strong> {{ results.environment.href }}</div>
        </div>
      </div>
      
      <!-- Test Results -->
      <div v-for="test in results.tests" :key="test.name" class="border rounded p-4">
        <h3 class="font-bold mb-2">{{ test.name }}</h3>
        <div class="space-y-2">
          <div 
            v-for="(result, index) in test.results" 
            :key="index"
            class="text-sm p-2 rounded"
            :class="result.success ? 'bg-green-100' : 'bg-red-100'"
          >
            <div class="font-mono">
              {{ formatTestResult(test.name, result) }}
            </div>
          </div>
        </div>
      </div>
      
      <!-- Analysis -->
      <div v-if="analysis" class="bg-blue-50 border border-blue-200 rounded p-4">
        <h3 class="font-bold mb-2">Analysis</h3>
        
        <div v-if="analysis.issues.length > 0" class="mb-3">
          <h4 class="font-semibold text-red-600">Issues Found:</h4>
          <ul class="list-disc list-inside text-sm">
            <li v-for="(issue, index) in analysis.issues" :key="index">{{ issue }}</li>
          </ul>
        </div>
        
        <div v-if="analysis.recommendations.length > 0">
          <h4 class="font-semibold text-blue-600">Recommendations:</h4>
          <ul class="list-disc list-inside text-sm">
            <li v-for="(rec, index) in analysis.recommendations" :key="index">{{ rec }}</li>
          </ul>
        </div>
        
        <div v-if="analysis.issues.length === 0" class="text-green-600">
          ✅ No issues detected. Network connectivity appears to be working correctly.
        </div>
      </div>
      
      <!-- Raw Results -->
      <details class="bg-gray-100 p-4 rounded">
        <summary class="cursor-pointer font-bold">Raw Results (JSON)</summary>
        <pre class="mt-2 text-xs overflow-auto">{{ JSON.stringify(results, null, 2) }}</pre>
      </details>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { runNetworkDiagnostics, analyzeDiagnosticResults } from '@/utils/network-diagnostic';

const running = ref(false);
const results = ref(null);
const analysis = ref(null);

async function runDiagnostics() {
  running.value = true;
  results.value = null;
  analysis.value = null;
  
  try {
    const diagnosticResults = await runNetworkDiagnostics();
    results.value = diagnosticResults;
    
    // Analyze results
    analysis.value = analyzeDiagnosticResults(diagnosticResults);
    } catch (error) {
    console.error('❌ Diagnostic error:', error);
    results.value = {
      error: error.message,
      timestamp: new Date().toISOString()
    };
  } finally {
    running.value = false;
  }
}

function formatTestResult(testName, result) {
  switch (testName) {
    case 'Basic Connectivity':
      return `${result.endpoint} - ${result.success ? '✅ Connected' : '❌ Failed'} ${result.status ? `(${result.status})` : ''}`;
    
    case 'DNS Resolution':
      return `${result.hostname} - ${result.success ? '✅ Resolved' : '❌ Failed'}`;
    
    case 'Port Accessibility':
      return `Port ${result.port} - ${result.accessible ? '✅ Accessible' : '❌ Not accessible'}`;
    
    case 'CORS Headers':
      return result.success 
        ? `✅ CORS enabled for ${result.origin}` 
        : `❌ CORS not configured for ${result.origin}`;
    
    case 'Request Methods':
      return `${result.method} ${result.endpoint || ''} - ${result.success ? '✅' : '❌'} ${result.status ? `(${result.status})` : ''}`;
    
    case 'Content Types':
      return `${result.contentType} - ${result.acceptsType ? '✅ Accepted' : '❌ Not accepted'}`;
    
    default:
      return JSON.stringify(result);
  }
}
</script>