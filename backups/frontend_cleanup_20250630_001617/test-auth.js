const fetch = require('node-fetch');

async function testAuth() {
  console.log('🔐 Testing authentication and file download...');

  try {
    // Test 1: Login with provided credentials
    console.log('\n1️⃣ Testing login...');
    const loginResponse = await fetch('http://localhost:5174/api/signin', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        email: 'super@test.com',
        password: 'password'
      })
    });

    console.log('Login Status:', loginResponse.status, loginResponse.statusText);

    if (!loginResponse.ok) {
      const loginError = await loginResponse.text();
      console.log('Login Error:', loginError);
      return;
    }

    const loginData = await loginResponse.json();
    console.log('✅ Login successful!');
    console.log('Token preview:', loginData.token ? loginData.token.substring(0, 20) + '...' : 'No token');

    if (!loginData.token) {
      console.log('❌ No token in login response');
      return;
    }

    // Test 2: File download with authentication
    console.log('\n2️⃣ Testing file download with auth...');
    const fileUrl = '/api/files/download/60c155658fcb1ef14145b5c9e359a571c504b8e1a7449d9965f720d3c1eebb68.png';

    const fileResponse = await fetch('http://localhost:5174' + fileUrl, {
      headers: {
        'Authorization': `Bearer ${loginData.token}`
      }
    });

    console.log('File Download Status:', fileResponse.status, fileResponse.statusText);
    console.log('Content-Type:', fileResponse.headers.get('content-type'));
    console.log('Content-Length:', fileResponse.headers.get('content-length'));

    if (!fileResponse.ok) {
      const errorText = await fileResponse.text();
      console.log('❌ File download error:', errorText);

      // Test 3: Try a simpler filename
      console.log('\n3️⃣ Testing with simple filename...');
      const simpleResponse = await fetch('http://localhost:5174/api/files/download/test.png', {
        headers: {
          'Authorization': `Bearer ${loginData.token}`
        }
      });

      console.log('Simple file Status:', simpleResponse.status, simpleResponse.statusText);
      if (!simpleResponse.ok) {
        const simpleError = await simpleResponse.text();
        console.log('Simple file error:', simpleError);
      }
    } else {
      console.log('✅ File download successful!');
      console.log('Response size:', fileResponse.headers.get('content-length'), 'bytes');
    }

  } catch (error) {
    console.error('❌ Test failed:', error.message);
  }
}

testAuth(); 