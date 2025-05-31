// 测试workspace切换功能
const baseUrl = 'http://127.0.0.1:6688/api';

// 解析JWT token中的用户信息
function parseJWTPayload(token) {
  try {
    const base64Payload = token.split('.')[1];
    const payload = Buffer.from(base64Payload, 'base64').toString('utf-8');
    return JSON.parse(payload);
  } catch (error) {
    throw new Error('无法解析JWT token');
  }
}

async function testWorkspaceSwitch() {
  try {
    console.log('🔐 正在登录...');
    
    // 1. 登录获取token
    const loginResponse = await fetch(`${baseUrl}/signin`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: 'testuser@example.com',
        password: 'password123'
      })
    });
    
    if (!loginResponse.ok) {
      throw new Error(`登录失败: ${loginResponse.status}`);
    }
    
    const loginData = await loginResponse.json();
    const token = loginData.access_token;
    
    // 解析token获取用户信息
    const tokenPayload = parseJWTPayload(token);
    const currentUser = tokenPayload.user;
    
    console.log('✅ 登录成功');
    console.log('👤 当前用户:', currentUser.email);
    console.log('📍 当前workspace ID:', currentUser.workspace_id);
    
    // 2. 获取所有可用的workspace
    console.log('\n📋 获取所有workspace...');
    const workspacesResponse = await fetch(`${baseUrl}/workspaces`, {
      headers: { 'Authorization': `Bearer ${token}` }
    });
    
    if (!workspacesResponse.ok) {
      throw new Error(`获取workspace列表失败: ${workspacesResponse.status}`);
    }
    
    const workspaces = await workspacesResponse.json();
    console.log('✅ 找到的workspace:');
    workspaces.forEach((ws, index) => {
      console.log(`   ${index + 1}. ${ws.name} (ID: ${ws.id}) - Owner: ${ws.owner_id}`);
    });
    
    // 3. 选择一个不同的workspace来切换
    const currentWorkspaceId = currentUser.workspace_id;
    const targetWorkspace = workspaces.find(ws => ws.id !== currentWorkspaceId);
    
    if (!targetWorkspace) {
      console.log('⚠️ 没有找到其他可切换的workspace');
      return;
    }
    
    console.log(`\n🔄 切换到workspace: ${targetWorkspace.name} (ID: ${targetWorkspace.id})`);
    
    // 4. 切换workspace
    const switchResponse = await fetch(`${baseUrl}/user/switch-workspace`, {
      method: 'POST',
      headers: { 
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}` 
      },
      body: JSON.stringify({
        workspace_id: targetWorkspace.id
      })
    });
    
    if (!switchResponse.ok) {
      const errorText = await switchResponse.text();
      throw new Error(`切换workspace失败: ${switchResponse.status} - ${errorText}`);
    }
    
    const switchResult = await switchResponse.json();
    console.log('✅ 切换成功!');
    console.log('📝 响应消息:', switchResult.message);
    console.log('👤 新用户信息:', {
      id: switchResult.user.id,
      email: switchResult.user.email,
      workspace_id: switchResult.user.workspace_id
    });
    console.log('🏢 新workspace信息:', {
      id: switchResult.workspace.id,
      name: switchResult.workspace.name,
      owner_id: switchResult.workspace.owner_id
    });
    
    // 5. 验证切换是否成功 - 重新登录验证
    console.log('\n🔍 验证切换结果...');
    const verifyLoginResponse = await fetch(`${baseUrl}/signin`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: 'testuser@example.com',
        password: 'password123'
      })
    });
    
    if (verifyLoginResponse.ok) {
      const verifyData = await verifyLoginResponse.json();
      const verifyPayload = parseJWTPayload(verifyData.access_token);
      const verifyUser = verifyPayload.user;
      
      console.log('✅ 验证结果:');
      console.log(`   原workspace: ${currentWorkspaceId}`);
      console.log(`   新workspace: ${verifyUser.workspace_id}`);
      
      if (verifyUser.workspace_id === targetWorkspace.id) {
        console.log('🎉 Workspace切换测试成功!');
      } else {
        console.log('❌ Workspace切换验证失败');
      }
    }
    
  } catch (error) {
    console.error('❌ 测试失败:', error.message);
  }
}

// 运行测试
testWorkspaceSwitch(); 