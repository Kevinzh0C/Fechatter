// Network diagnostic utility to identify connection issues

export async function runNetworkDiagnostics() {
  const results = {
    timestamp: new Date().toISOString(),
    environment: {
      userAgent: navigator.userAgent,
      onLine: navigator.onLine,
      protocol: window.location.protocol,
      hostname: window.location.hostname,
      port: window.location.port,
      href: window.location.href
    },
    tests: []
  };

  // Test 1: Basic connectivity
  results.tests.push(await testBasicConnectivity());

  // Test 2: DNS resolution (using different hostnames)
  results.tests.push(await testDNSResolution());

  // Test 3: Port accessibility
  results.tests.push(await testPortAccessibility());

  // Test 4: CORS headers
  results.tests.push(await testCORSHeaders());

  // Test 5: Request methods
  results.tests.push(await testRequestMethods());

  // Test 6: Content types
  results.tests.push(await testContentTypes());

  return results;
}

async function testBasicConnectivity() {
  const test = {
    name: 'Basic Connectivity',
    results: []
  };

  // Test direct health endpoint
  try {
    const response = await fetch('http://localhost:8080/health', {
      method: 'GET',
      mode: 'cors'
    });
    test.results.push({
      endpoint: 'http://localhost:8080/health',
      success: response.ok,
      status: response.status,
      headers: Object.fromEntries(response.headers.entries())
    });
  } catch (error) {
    test.results.push({
      endpoint: 'http://localhost:8080/health',
      success: false,
      error: error.message
    });
  }

  return test;
}

async function testDNSResolution() {
  const test = {
    name: 'DNS Resolution',
    results: []
  };

  const hostnames = [
    'localhost',
    '127.0.0.1',
    '[::1]', // IPv6 localhost
    '0.0.0.0'
  ];

  for (const hostname of hostnames) {
    try {
      const url = `http://${hostname}:8080/health`;
      const response = await fetch(url, {
        method: 'GET',
        mode: 'cors',
        signal: AbortSignal.timeout(3000)
      });
      
      test.results.push({
        hostname,
        url,
        success: response.ok,
        status: response.status
      });
    } catch (error) {
      test.results.push({
        hostname,
        url: `http://${hostname}:8080/health`,
        success: false,
        error: error.message
      });
    }
  }

  return test;
}

async function testPortAccessibility() {
  const test = {
    name: 'Port Accessibility',
    results: []
  };

  // Create an image element to test if port is accessible
  const testPort = (port) => {
    return new Promise((resolve) => {
      const img = new Image();
      const timeout = setTimeout(() => {
        img.onload = img.onerror = null;
        resolve({ port, accessible: false, method: 'image' });
      }, 2000);

      img.onload = () => {
        clearTimeout(timeout);
        resolve({ port, accessible: true, method: 'image' });
      };

      img.onerror = () => {
        clearTimeout(timeout);
        resolve({ port, accessible: false, method: 'image' });
      };

      img.src = `http://localhost:${port}/favicon.ico?t=${Date.now()}`;
    });
  };

  test.results.push(await testPort(8080));
  test.results.push(await testPort(1420));

  return test;
}

async function testCORSHeaders() {
  const test = {
    name: 'CORS Headers',
    results: []
  };

  try {
    const response = await fetch('http://localhost:8080/api/signin', {
      method: 'OPTIONS',
      headers: {
        'Origin': window.location.origin,
        'Access-Control-Request-Method': 'POST',
        'Access-Control-Request-Headers': 'content-type'
      }
    });

    const corsHeaders = {};
    for (const [key, value] of response.headers.entries()) {
      if (key.toLowerCase().startsWith('access-control')) {
        corsHeaders[key] = value;
      }
    }

    test.results.push({
      origin: window.location.origin,
      status: response.status,
      corsHeaders,
      success: response.status === 200 && corsHeaders['access-control-allow-origin'] !== undefined
    });
  } catch (error) {
    test.results.push({
      origin: window.location.origin,
      success: false,
      error: error.message
    });
  }

  return test;
}

async function testRequestMethods() {
  const test = {
    name: 'Request Methods',
    results: []
  };

  const methods = ['GET', 'POST', 'OPTIONS'];
  
  for (const method of methods) {
    try {
      const endpoint = method === 'POST' ? '/api/signin' : '/health';
      const response = await fetch(`http://localhost:8080${endpoint}`, {
        method,
        headers: {
          'Content-Type': 'application/json'
        },
        body: method === 'POST' ? JSON.stringify({ test: true }) : undefined,
        signal: AbortSignal.timeout(3000)
      });

      test.results.push({
        method,
        endpoint,
        success: response.ok || response.status < 500,
        status: response.status
      });
    } catch (error) {
      test.results.push({
        method,
        success: false,
        error: error.message
      });
    }
  }

  return test;
}

async function testContentTypes() {
  const test = {
    name: 'Content Types',
    results: []
  };

  const contentTypes = [
    'application/json',
    'application/x-www-form-urlencoded',
    'text/plain'
  ];

  for (const contentType of contentTypes) {
    try {
      const body = contentType === 'application/json' 
        ? JSON.stringify({ email: 'test@test.com', password: 'test' })
        : 'email=test@test.com&password=test';

      const response = await fetch('http://localhost:8080/api/signin', {
        method: 'POST',
        headers: {
          'Content-Type': contentType
        },
        body,
        signal: AbortSignal.timeout(3000)
      });

      test.results.push({
        contentType,
        success: response.status !== 0, // 0 means network error
        status: response.status,
        acceptsType: response.status !== 415 // 415 = Unsupported Media Type
      });
    } catch (error) {
      test.results.push({
        contentType,
        success: false,
        error: error.message
      });
    }
  }

  return test;
}

// Analyze results and provide recommendations
export function analyzeDiagnosticResults(results) {
  const analysis = {
    issues: [],
    recommendations: []
  };

  // Check basic connectivity
  const basicTest = results.tests.find(t => t.name === 'Basic Connectivity');
  if (basicTest && !basicTest.results[0]?.success) {
    analysis.issues.push('Cannot connect to backend server');
    analysis.recommendations.push('Ensure the backend server is running on port 8080');
  }

  // Check DNS resolution
  const dnsTest = results.tests.find(t => t.name === 'DNS Resolution');
  if (dnsTest) {
    const workingHosts = dnsTest.results.filter(r => r.success);
    if (workingHosts.length === 0) {
      analysis.issues.push('No hostname can reach the backend');
      analysis.recommendations.push('Check firewall settings and ensure port 8080 is open');
    } else if (workingHosts.length < dnsTest.results.length) {
      analysis.issues.push(`Only ${workingHosts.map(h => h.hostname).join(', ')} can reach the backend`);
      analysis.recommendations.push(`Use ${workingHosts[0].hostname} instead of localhost`);
    }
  }

  // Check CORS
  const corsTest = results.tests.find(t => t.name === 'CORS Headers');
  if (corsTest && !corsTest.results[0]?.success) {
    analysis.issues.push('CORS is not properly configured');
    analysis.recommendations.push('Check backend CORS configuration for origin ' + window.location.origin);
  }

  return analysis;
}

export default {
  runNetworkDiagnostics,
  analyzeDiagnosticResults
};