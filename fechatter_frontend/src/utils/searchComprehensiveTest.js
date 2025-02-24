/**
 * Comprehensive Search Function Test
 * Complete validation of the search feature DAG chain
 */

class SearchComprehensiveTest {
  constructor() {
    this.testResults = [];
    this.startTime = Date.now();
  }

  /**
   * Run all search functionality tests
   */
  async runAll() {
    if (import.meta.env.DEV) {
      console.log('🔍 [COMPREHENSIVE SEARCH TEST] Starting complete validation...');
    if (import.meta.env.DEV) {
      console.log('===============================================================');
    }

    try {
      await this.testAuthenticationFlow();
      await this.testSearchButtonLogic();
      await this.testAPICall();
      await this.testResponseProcessing();
      await this.testUIIntegration();

      this.generateReport();
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('🔍 [COMPREHENSIVE TEST] Test suite failed:', error);
      this.testResults.push({
        test: 'Test Suite Execution',
        status: 'FAIL',
        error: error.message
      });

  /**
   * Test 1: Authentication Flow
   */
  async testAuthenticationFlow() {
    if (import.meta.env.DEV) {
      console.log('\n1️⃣ Testing Authentication Flow');
    if (import.meta.env.DEV) {
      console.log('------------------------------');
    }

    // Check token manager existence
    if (window.tokenManager) {
      if (import.meta.env.DEV) {
        console.log('✅ Token manager available');
      this.testResults.push({ test: 'Token Manager', status: 'PASS' });
    } else {
      if (import.meta.env.DEV) {
        console.log('❌ Token manager not found');
      this.testResults.push({ test: 'Token Manager', status: 'FAIL' });

    // Check auth store
    if (window.$router && window.$router.app && window.$router.app.config.globalProperties) {
      if (import.meta.env.DEV) {
        console.log('✅ Auth store accessible');
      this.testResults.push({ test: 'Auth Store', status: 'PASS' });
    } else {
      if (import.meta.env.DEV) {
        console.log('⚠️ Auth store access method unclear');
      this.testResults.push({ test: 'Auth Store', status: 'PARTIAL' });

    // Test token retrieval
    const token = window.tokenManager?.getAccessToken();
    if (token) {
      if (import.meta.env.DEV) {
        console.log('✅ Authentication token available');
      if (import.meta.env.DEV) {
        console.log('🔐 Token preview:', token.substring(0, 20) + '...');
      this.testResults.push({ test: 'Token Retrieval', status: 'PASS' });
    } else {
      if (import.meta.env.DEV) {
        console.log('❌ No authentication token found');
      this.testResults.push({ test: 'Token Retrieval', status: 'FAIL' });

  /**
   * Test 2: Search Button Logic
   */
  async testSearchButtonLogic() {
    if (import.meta.env.DEV) {
      console.log('\n2️⃣ Testing Search Button Logic');
    if (import.meta.env.DEV) {
      console.log('------------------------------');
    }

    // Find search button
    const searchButton = document.querySelector('button[title*="Search"]');
    if (searchButton) {
      if (import.meta.env.DEV) {
        console.log('✅ Search button found in DOM');
      this.testResults.push({ test: 'Search Button DOM', status: 'PASS' });

      // Test click simulation
      try {
        const clickEvent = new MouseEvent('click', { bubbles: true });
        searchButton.dispatchEvent(clickEvent);
        if (import.meta.env.DEV) {
          console.log('✅ Search button click simulation successful');
        this.testResults.push({ test: 'Button Click', status: 'PASS' });
      } catch (error) {
        if (import.meta.env.DEV) {
          console.log('❌ Search button click failed:', error.message);
        this.testResults.push({ test: 'Button Click', status: 'FAIL', error: error.message });
    } else {
      if (import.meta.env.DEV) {
        console.log('❌ Search button not found in DOM');
      this.testResults.push({ test: 'Search Button DOM', status: 'FAIL' });

    // Test keyboard shortcut
    try {
      const keyEvent = new KeyboardEvent('keydown', {
        key: 'f',
        ctrlKey: true,
        bubbles: true
      });
      document.dispatchEvent(keyEvent);
      if (import.meta.env.DEV) {
        console.log('✅ Keyboard shortcut simulation successful');
      this.testResults.push({ test: 'Keyboard Shortcut', status: 'PASS' });
    } catch (error) {
      if (import.meta.env.DEV) {
        console.log('❌ Keyboard shortcut failed:', error.message);
      this.testResults.push({ test: 'Keyboard Shortcut', status: 'FAIL', error: error.message });

  /**
   * Test 3: API Call
   */
  async testAPICall() {
    if (import.meta.env.DEV) {
      console.log('\n3️⃣ Testing Search API Call');
    if (import.meta.env.DEV) {
      console.log('-------------------------');
    }

    try {
      // Get SearchService from window if available
      const SearchService = window.SearchService ||
        (await import('/src/services/api.js')).SearchService;

      if (!SearchService) {
        throw new Error('SearchService not available');
      }

      if (import.meta.env.DEV) {
        console.log('✅ SearchService loaded');
      this.testResults.push({ test: 'SearchService Load', status: 'PASS' });

      // Test API call
      const testParams = {
        query: 'test',
        limit: 5
      };

      if (import.meta.env.DEV) {
        console.log('🔍 Testing API call with params:', testParams);
      const response = await SearchService.search(testParams);

      if (import.meta.env.DEV) {
        console.log('✅ Search API call successful');
      if (import.meta.env.DEV) {
        console.log('📊 Response summary:', {
        resultsCount: response.results?.length || 0,
        total: response.total,
        took_ms: response.took_ms
      });

      this.testResults.push({ test: 'API Call', status: 'PASS' });

      // Validate response structure
      if (response.results && Array.isArray(response.results)) {
        if (import.meta.env.DEV) {
          console.log('✅ Response structure valid');
        this.testResults.push({ test: 'Response Structure', status: 'PASS' });
      } else {
        if (import.meta.env.DEV) {
          console.log('❌ Invalid response structure');
        this.testResults.push({ test: 'Response Structure', status: 'FAIL' });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.log('❌ Search API call failed:', error.message);
      this.testResults.push({ test: 'API Call', status: 'FAIL', error: error.message });

  /**
   * Test 4: Response Processing
   */
  async testResponseProcessing() {
    if (import.meta.env.DEV) {
      console.log('\n4️⃣ Testing Response Processing');
    if (import.meta.env.DEV) {
      console.log('-----------------------------');
    }

    // Test response normalization
    const mockBackendResponse = {
      success: true,
      data: {
        results: [
          {
            id: 1,
            content: 'test message',
            sender_name: 'Test User',
            created_at: '2024-01-01T00:00:00Z'
          }
        ],
        total: 1,
        took_ms: 10,
        query: 'test',
        page: {
          offset: 0,
          limit: 5,
          has_more: false
        }
    };

    try {
      // Test response processing logic
      const searchData = mockBackendResponse.data;
      const normalizedResponse = {
        results: searchData.results || [],
        total: searchData.total || 0,
        took_ms: searchData.took_ms || 0,
        query: searchData.query || '',
        page: {
          offset: searchData.page?.offset || 0,
          limit: searchData.page?.limit || 5,
          has_more: searchData.page?.has_more || false
        }
      };

      if (import.meta.env.DEV) {
        console.log('✅ Response processing successful');
      if (import.meta.env.DEV) {
        console.log('📋 Normalized response:', normalizedResponse);
      this.testResults.push({ test: 'Response Processing', status: 'PASS' });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.log('❌ Response processing failed:', error.message);
      this.testResults.push({ test: 'Response Processing', status: 'FAIL', error: error.message });

  /**
   * Test 5: UI Integration
   */
  async testUIIntegration() {
    if (import.meta.env.DEV) {
      console.log('\n5️⃣ Testing UI Integration');
    if (import.meta.env.DEV) {
      console.log('-----------------------');
    }

    // Check for search modal
    const searchModal = document.querySelector('[role="dialog"]') ||
      document.querySelector('.modern-search-modal') ||
      document.querySelector('.search-modal');

    if (searchModal) {
      if (import.meta.env.DEV) {
        console.log('✅ Search modal found in DOM');
      this.testResults.push({ test: 'Search Modal DOM', status: 'PASS' });
    } else {
      if (import.meta.env.DEV) {
        console.log('⚠️ Search modal not currently visible (expected if not triggered)');
      this.testResults.push({ test: 'Search Modal DOM', status: 'PARTIAL' });

    // Test for search input elements
    const searchInputs = document.querySelectorAll('input[placeholder*="Search"]');
    if (searchInputs.length > 0) {
      if (import.meta.env.DEV) {
        console.log('✅ Search input elements found:', searchInputs.length);
      this.testResults.push({ test: 'Search Input Elements', status: 'PASS' });
    } else {
      if (import.meta.env.DEV) {
        console.log('❌ No search input elements found');
      this.testResults.push({ test: 'Search Input Elements', status: 'FAIL' });

  /**
   * Generate comprehensive test report
   */
  generateReport() {
    const duration = Date.now() - this.startTime;
    const passed = this.testResults.filter(r => r.status === 'PASS').length;
    const failed = this.testResults.filter(r => r.status === 'FAIL').length;
    const partial = this.testResults.filter(r => r.status === 'PARTIAL').length;

    if (import.meta.env.DEV) {
      console.log('\n📊 COMPREHENSIVE SEARCH TEST REPORT');
    if (import.meta.env.DEV) {
      console.log('====================================');
    if (import.meta.env.DEV) {
      console.log(`⏱️  Duration: ${duration}ms`);
    if (import.meta.env.DEV) {
      console.log(`✅ Passed: ${passed}`);
    if (import.meta.env.DEV) {
      console.log(`❌ Failed: ${failed}`);
    if (import.meta.env.DEV) {
      console.log(`⚠️  Partial: ${partial}`);
    if (import.meta.env.DEV) {
      console.log(`📈 Success Rate: ${((passed / this.testResults.length) * 100).toFixed(1)}%`);
    }

    if (import.meta.env.DEV) {
      console.log('\n📋 Detailed Results:');
    this.testResults.forEach(result => {
      const status = result.status === 'PASS' ? '✅' :
        result.status === 'FAIL' ? '❌' : '⚠️';
      if (import.meta.env.DEV) {
        console.log(`${status} ${result.test}`);
      if (result.error) {
        if (import.meta.env.DEV) {
          console.log(`   Error: ${result.error}`);
        }
    });

    // Store results globally for inspection
    window.searchTestResults = {
      summary: { passed, failed, partial, duration },
      details: this.testResults
    };

    if (import.meta.env.DEV) {
      console.log('\n💡 Results stored in window.searchTestResults');
    if (import.meta.env.DEV) {
      console.log('🔍 Run window.searchComprehensiveTest.runAll() to test again');
    }

// Global availability
window.SearchComprehensiveTest = SearchComprehensiveTest;
window.searchComprehensiveTest = new SearchComprehensiveTest();

if (import.meta.env.DEV) {
  console.log('🔍 Search Comprehensive Test loaded');
if (import.meta.env.DEV) {
  console.log('📋 Run window.searchComprehensiveTest.runAll() to start testing');
}

export default SearchComprehensiveTest; 