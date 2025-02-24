/**
 * Minimal Search Test - 绕过所有复杂逻辑
 */

async function minimalSearchTest() {
  if (import.meta.env.DEV) {
    console.log('🧪 [MINIMAL] Starting minimal search test...');
  }

  try {
    // 获取token
    const token = localStorage.getItem('auth_token') || localStorage.getItem('token');
    if (!token) {
      if (import.meta.env.DEV) {
        console.error('❌ No token found');
      return;
    }

    if (import.meta.env.DEV) {
      console.log('🧪 [MINIMAL] Token found, testing direct fetch...');
    }

    // 使用最基本的fetch，无任何额外处理
    const response = await fetch('/api/search/messages?q=Hi&limit=5', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Accept': 'application/json',
        'Cache-Control': 'no-cache'
      }
    });

    if (import.meta.env.DEV) {
      console.log('🧪 [MINIMAL] Response received:');
    if (import.meta.env.DEV) {
      console.log('  Status:', response.status);
    if (import.meta.env.DEV) {
      console.log('  Content-Type:', response.headers.get('content-type'));
    if (import.meta.env.DEV) {
      console.log('  Content-Length:', response.headers.get('content-length'));
    }

    // 先获取原始文本
    const textResponse = await response.text();
    if (import.meta.env.DEV) {
      console.log('🧪 [MINIMAL] Raw response text:');
    if (import.meta.env.DEV) {
      console.log('  Length:', textResponse.length);
    if (import.meta.env.DEV) {
      console.log('  First 200 chars:', textResponse.substring(0, 200));
    }

    // 尝试解析JSON
    try {
      const jsonResponse = JSON.parse(textResponse);
      if (import.meta.env.DEV) {
        console.log('🧪 [MINIMAL] Parsed JSON:');
      if (import.meta.env.DEV) {
        console.log('  Type:', typeof jsonResponse);
      if (import.meta.env.DEV) {
        console.log('  Keys:', Object.keys(jsonResponse));
      if (import.meta.env.DEV) {
        console.log('  Full object:', jsonResponse);
      }

      // 检查预期结构
      if (jsonResponse.success && jsonResponse.data) {
        if (import.meta.env.DEV) {
          console.log('✅ [MINIMAL] Expected structure found!');
        if (import.meta.env.DEV) {
          console.log('  Results count:', jsonResponse.data.results?.length || 0);
        }
      } else {
        if (import.meta.env.DEV) {
          console.warn('⚠️ [MINIMAL] Unexpected structure');
        }

    } catch (parseError) {
      if (import.meta.env.DEV) {
        console.error('❌ [MINIMAL] JSON parse failed:', parseError);
      if (import.meta.env.DEV) {
        console.log('Raw response was:', textResponse);
      }

  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('❌ [MINIMAL] Test failed:', error);
    }

// 立即可用
if (typeof window !== 'undefined') {
  window.minimalSearchTest = minimalSearchTest;
  if (import.meta.env.DEV) {
    console.log('🧪 Minimal search test available: window.minimalSearchTest()');
  }

export default minimalSearchTest; 