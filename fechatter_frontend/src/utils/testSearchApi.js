/**
 * Test Search API - Simple direct test
 */

async function testSearchApi() {
  if (import.meta.env.DEV) {
    console.log('🔍 Testing Search API...');
  }

  try {
    // Direct fetch to see raw response
    const response = await fetch('/api/search/messages', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'Authorization': `Bearer ${localStorage.getItem('auth_token') || ''}`
      },
      body: JSON.stringify({
        q: 'test',
        limit: 5
      })
    });

    if (import.meta.env.DEV) {
      console.log('📡 Response Status:', response.status);
    if (import.meta.env.DEV) {
      console.log('📡 Response Headers:', Object.fromEntries(response.headers.entries()));
    }

    const text = await response.text();
    if (import.meta.env.DEV) {
      console.log('📡 Raw Response Text:', text);
    }

    try {
      const json = JSON.parse(text);
      if (import.meta.env.DEV) {
        console.log('📡 Parsed JSON:', json);
      if (import.meta.env.DEV) {
        console.log('📡 JSON Structure:', {
        type: typeof json,
        isArray: Array.isArray(json),
        keys: Object.keys(json || {}),
        hasSuccess: 'success' in json,
        hasData: 'data' in json,
        hasHits: 'hits' in json,
        hasResults: 'results' in json
      });

      // Show what frontend expects vs what we got
      if (import.meta.env.DEV) {
        console.log('🔍 Frontend expects one of:');
      if (import.meta.env.DEV) {
        console.log('   1. { success: true, data: { hits: [...] } }');
      if (import.meta.env.DEV) {
        console.log('   2. { hits: [...] }');
      if (import.meta.env.DEV) {
        console.log('   3. { results: [...] }');
      if (import.meta.env.DEV) {
        console.log('   4. [...]');
      if (import.meta.env.DEV) {
        console.log('🔍 But we got:', Object.keys(json || {}));
      }

    } catch (e) {
      if (import.meta.env.DEV) {
        console.error('❌ Failed to parse as JSON:', e);
      }

  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('❌ Request failed:', error);
    }

// Add to window
window.testSearchApi = testSearchApi;

if (import.meta.env.DEV) {
  console.log('🔍 Search API Test loaded. Run: window.testSearchApi()');
}

export { testSearchApi }; 