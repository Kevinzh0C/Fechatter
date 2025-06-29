/**
 * Direct Search Test - Bypasses all layers to test search directly
 */

async function directSearchTest() {
  if (import.meta.env.DEV) {
    console.log('[DIRECT SEARCH TEST] Starting...');
  }

  try {
    // Get token
    const token = window.tokenManager?.getAccessToken() ||
      localStorage.getItem('auth_token') ||
      localStorage.getItem('token');

    if (!token) {
      if (import.meta.env.DEV) {
        console.error('ERROR: No token found. Please login first.');
      return;
    }

    if (import.meta.env.DEV) {
      console.log('🔐 Using token:', token.substring(0, 20) + '...');
    }

    // Test 1: Direct fetch to backend
    if (import.meta.env.DEV) {
      console.log('\n1️⃣ Testing direct fetch to backend...');
    try {
      const backendResponse = await fetch('http://45.77.178.85:8080/api/search/messages?q=test&limit=5', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Accept': 'application/json'
        }
      });

      const backendData = await backendResponse.json();
      if (import.meta.env.DEV) {
        console.log('Backend response:', backendData);
      }
    } catch (e) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Backend direct test failed:', e);
      }

    // Test 2: Through Vite proxy
    if (import.meta.env.DEV) {
      console.log('\n2️⃣ Testing through Vite proxy...');
    try {
      const proxyResponse = await fetch('/api/search/messages?q=test&limit=5', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Accept': 'application/json'
        }
      });

      const proxyData = await proxyResponse.json();
      if (import.meta.env.DEV) {
        console.log('Proxy response:', proxyData);
      }
    } catch (e) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Proxy test failed:', e);
      }

    // Test 3: Using axios directly
    if (import.meta.env.DEV) {
      console.log('\n3️⃣ Testing with axios directly...');
    try {
      const axios = window.axios || (await import('axios')).default;
      const axiosResponse = await axios.post('/api/search/messages?q=test&limit=5', null, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Accept': 'application/json'
        }
      });

      if (import.meta.env.DEV) {
        console.log('Axios response:', axiosResponse.data);
      }
    } catch (e) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Axios test failed:', e);
      if (e.response) {
        if (import.meta.env.DEV) {
          console.error('Response data:', e.response.data);
        if (import.meta.env.DEV) {
          console.error('Response status:', e.response.status);
        }

    // Test 4: Using the SearchService
    if (import.meta.env.DEV) {
      console.log('\n4️⃣ Testing with SearchService...');
    try {
      const { SearchService } = await import('../services/api.js');
      const serviceResponse = await SearchService.search({
        query: 'test',
        limit: 5
      });

      if (import.meta.env.DEV) {
        console.log('SearchService response:', serviceResponse);
      }
    } catch (e) {
      if (import.meta.env.DEV) {
        console.error('ERROR: SearchService test failed:', e);
      }

  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('ERROR: Test suite failed:', error);
    }

  if (import.meta.env.DEV) {
    console.log('\n[DIRECT SEARCH TEST] Complete');
  }

// Make available globally
window.directSearchTest = directSearchTest;

if (import.meta.env.DEV) {
  console.log('Direct Search Test loaded. Run window.directSearchTest()');
}

export default directSearchTest; 