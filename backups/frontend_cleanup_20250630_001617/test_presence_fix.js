/**
 * Test script to verify the presence endpoint fix
 * Run this in the browser console at http://localhost:1420
 */

async function testPresenceEndpoint() {
  try {
    console.log('🧪 Testing presence endpoint fix...');

    // Get the auth token (assuming user is logged in)
    const token = localStorage.getItem('auth_token') || sessionStorage.getItem('auth_token');

    if (!token) {
      console.error('❌ No auth token found. Please log in first.');
      return false;
    }

    // Test the corrected endpoint
    const response = await fetch('/api/realtime/presence', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`
      },
      body: JSON.stringify({
        status: 'online',
        timestamp: new Date().toISOString(),
        device_info: {
          userAgent: navigator.userAgent,
          platform: navigator.platform,
          language: navigator.language
        }
      })
    });

    console.log(`📡 Response status: ${response.status} ${response.statusText}`);

    if (response.ok) {
      const data = await response.json();
      console.log('✅ Presence endpoint is working!', data);
      return true;
    } else if (response.status === 401) {
      console.log('🔐 Endpoint exists but requires valid authentication');
      return true; // Endpoint exists, just auth issue
    } else {
      const errorText = await response.text();
      console.error(`❌ Unexpected response: ${response.status}`, errorText);
      return false;
    }

  } catch (error) {
    console.error('❌ Test failed:', error);
    return false;
  }
}

// Auto-run test if in browser context
if (typeof window !== 'undefined') {
  console.log('🔧 Presence endpoint test loaded. Run testPresenceEndpoint() to test.');

  // Wait a moment then auto-test
  setTimeout(() => {
    testPresenceEndpoint().then(success => {
      if (success) {
        console.log('🎉 Presence endpoint fix verification: SUCCESS');
      } else {
        console.log('❌ Presence endpoint fix verification: FAILED');
      }
    });
  }, 1000);
}

export { testPresenceEndpoint }; 