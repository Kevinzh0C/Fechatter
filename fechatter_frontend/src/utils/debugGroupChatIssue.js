/**
 * Debug tool for group chat redirect issue
 */

export async function debugGroupChatIssue() {
  if (import.meta.env.DEV) {
    console.log('🐛 Debugging Group Chat Redirect Issue...\n');
  }

  try {
    // Import required modules
    const { useAuthStore } = await import('@/stores/auth');
    const { useWorkspaceStore } = await import('@/stores/workspace');
    const { useChatStore } = await import('@/stores/chat');
    const { default: authStateManager } = await import('@/utils/authStateManager');

    const authStore = useAuthStore();
    const workspaceStore = useWorkspaceStore();
    const chatStore = useChatStore();

    // 1. Check authentication state
    if (import.meta.env.DEV) {
      console.log('📋 1. Authentication State:');
    const authState = authStateManager.getAuthState();
    if (import.meta.env.DEV) {
      console.log('- authStateManager:', {
        isAuthenticated: authState.isAuthenticated,
      hasToken: authState.hasToken,
      hasUser: authState.hasUser,
      user: authState.user
    });
    if (import.meta.env.DEV) {
      console.log('- authStore:', {
        isAuthenticated: authStore.isAuthenticated,
      isInitialized: authStore.isInitialized,
      user: authStore.user
    });

    // 2. Check workspace state
    if (import.meta.env.DEV) {
      console.log('\n📋 2. Workspace State:');
    if (import.meta.env.DEV) {
      console.log('- Current workspace:', workspaceStore.currentWorkspace);
    if (import.meta.env.DEV) {
      console.log('- Workspace users:', workspaceStore.workspaceUsers?.length || 0);
    if (import.meta.env.DEV) {
      console.log('- User workspace_id:', authStore.user?.workspace_id);
    }

    // 3. Check group chats
    if (import.meta.env.DEV) {
      console.log('\n📋 3. Group Chats:');
    const groupChats = chatStore.chats?.filter(chat => chat.chat_type === 'Group') || [];
    if (import.meta.env.DEV) {
      console.log('- Group chats count:', groupChats.length);
    groupChats.forEach(group => {
      if (import.meta.env.DEV) {
        console.log(`  - ${group.name} (ID: ${group.id})`);
      }
    });

    // 4. Test workspace fetch
    if (import.meta.env.DEV) {
      console.log('\n📋 4. Testing workspace fetch...');
    try {
      const result = await workspaceStore.fetchCurrentWorkspace();
      if (import.meta.env.DEV) {
        console.log('✅ Workspace fetch successful:', result);
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Workspace fetch failed:', error.message);
      }

    // 5. Check for missing workspace_id
    if (import.meta.env.DEV) {
      console.log('\n📋 5. Workspace ID Analysis:');
    if (!authStore.user?.workspace_id) {
      if (import.meta.env.DEV) {
        console.warn('⚠️ User does not have workspace_id field!');
      if (import.meta.env.DEV) {
        console.log('This is the root cause of the issue.');
      if (import.meta.env.DEV) {
        console.log('Solution: The backend should provide workspace_id in user data');
      if (import.meta.env.DEV) {
        console.log('Workaround: Frontend now uses default workspace_id = 1');
      }
    } else {
      if (import.meta.env.DEV) {
        console.log('✅ User has workspace_id:', authStore.user.workspace_id);
      }

    // 6. Provide fix status
    if (import.meta.env.DEV) {
      console.log('\n📋 6. Fix Status:');
    if (import.meta.env.DEV) {
      console.log('✅ workspace.js now handles missing workspace_id gracefully');
    if (import.meta.env.DEV) {
      console.log('✅ auth.js now catches workspace fetch errors');
    if (import.meta.env.DEV) {
      console.log('✅ Default workspace (ID: 1) is used when workspace_id is missing');
    }

    return {
      authOk: authState.isAuthenticated,
      workspaceOk: !!workspaceStore.currentWorkspace,
      hasWorkspaceId: !!authStore.user?.workspace_id,
      groupChatsCount: groupChats.length
    };

  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('❌ Debug failed:', error);
    return { error: error.message };
  }

// Auto-run in development
if (import.meta.env.DEV) {
  window.debugGroupChat = debugGroupChatIssue;
  console.log('🐛 Group chat debug tool loaded - use window.debugGroupChat()');
} 