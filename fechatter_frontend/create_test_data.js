// Simple test data creation script
const baseUrl = 'http://127.0.0.1:6688/api';

async function createTestData() {
  try {
    console.log('Creating test data...');
    console.log('Checking server connection...');
    
    // First check if server is running
    try {
      const healthResponse = await fetch(`http://127.0.0.1:6688/health`);
      if (!healthResponse.ok) {
        throw new Error('Health check failed');
      }
      console.log('✅ Server is running');
    } catch (error) {
      console.error('❌ Server is not running or not accessible');
      console.error('Please start the backend server with: cd fechatter_server && cargo run');
      return;
    }
    
    // Login to get token
    let token;
    const loginResponse = await fetch(`${baseUrl}/signin`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: 'testuser@example.com',
        password: 'password123'
      })
    });
    
    if (!loginResponse.ok) {
      const errorText = await loginResponse.text();
      console.error('❌ Login failed:', errorText);
      console.log('Trying to register a new user...');
      
      // Try to register
      const registerResponse = await fetch(`${baseUrl}/signup`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: 'testuser@example.com',
          password: 'password123',
          fullname: 'Test User',
          workspace: 'test-workspace'
        })
      });
      
      if (registerResponse.ok) {
        console.log('✅ User registered, now logging in...');
        const loginRetry = await fetch(`${baseUrl}/signin`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            email: 'testuser@example.com',
            password: 'password123'
          })
        });
        
        if (!loginRetry.ok) {
          console.error('❌ Login still failed after registration');
          return;
        }
        
        const loginData = await loginRetry.json();
        token = loginData.access_token;
        console.log('✅ Login successful after registration');
      } else {
        console.error('❌ Registration also failed');
        return;
      }
    } else {
      const loginData = await loginResponse.json();
      token = loginData.access_token;
      console.log('✅ Login successful');
    }
    
    // Create some test channels
    const channels = [
      { name: 'general', description: 'General discussion', chat_type: 'PublicChannel' },
      { name: 'random', description: 'Random chat', chat_type: 'PublicChannel' },
      { name: 'dev-team', description: 'Development team channel', chat_type: 'PrivateChannel' }
    ];
    
    for (const channel of channels) {
      const response = await fetch(`${baseUrl}/chats`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
          name: channel.name,
          description: channel.description,
          chat_type: channel.chat_type,
          member_ids: []
        })
      });
      
      if (response.ok) {
        const chatData = await response.json();
        console.log(`✅ Created channel: ${channel.name} (ID: ${chatData.id})`);
        
        // Add some test messages
        const messages = [
          `Welcome to #${channel.name}!`,
          `This is a test message in ${channel.name}`,
          `Let's start chatting here! 🎉`
        ];
        
        for (const content of messages) {
          await fetch(`${baseUrl}/chats/${chatData.id}/messages`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ content })
          });
        }
        
        console.log(`✅ Added messages to ${channel.name}`);
      } else {
        console.log(`❌ Failed to create channel: ${channel.name}`);
      }
    }
    
    console.log('🎉 Test data creation completed!');
    
  } catch (error) {
    console.error('Error creating test data:', error);
  }
}

// Run if called directly
if (typeof window === 'undefined') {
  createTestData();
}

// Export for browser use
if (typeof window !== 'undefined') {
  window.createTestData = createTestData;
} 