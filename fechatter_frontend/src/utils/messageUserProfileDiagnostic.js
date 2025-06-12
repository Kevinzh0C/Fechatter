/**
 * Message User Profile Diagnostic
 * Check why user profiles are not loading for messages
 */

async function diagnoseMessageUserProfiles() {
  console.group('👤 Message User Profile Diagnostic');

  try {
    const { useChatStore } = await import('@/stores/chat');
    const { useUserStore } = await import('@/stores/user');
    const { useRoute } = await import('vue-router');
    const chatStore = useChatStore();
    const userStore = useUserStore();
    const route = useRoute();

    console.log('\n1️⃣ Current Messages Analysis');
    console.log('  - Total messages:', chatStore.messages.length);

    // Check first 5 messages
    const sampleMessages = chatStore.messages.slice(0, 5);
    sampleMessages.forEach((msg, index) => {
      console.log(`\n  Message ${index + 1}:`, {
        id: msg.id,
        sender_id: msg.sender_id,
        sender: msg.sender,
        content: msg.content?.substring(0, 50) + '...'
      });
    });

    console.log('\n2️⃣ User Store Analysis');
    console.log('  - Workspace users loaded:', userStore.workspaceUsers.length);
    console.log('  - Sample users:', userStore.workspaceUsers.slice(0, 3));

    console.log('\n3️⃣ API Response Check');

    if (route.params.id) {
      const chatId = parseInt(route.params.id, 10);
      const { default: api } = await import('@/services/api');

      try {
        const response = await api.get(`/chat/${chatId}/messages`, {
          params: { limit: 5 }
        });

        console.log('  Raw API response structure:', {
          hasData: !!response.data,
          hasDataData: !!response.data?.data,
          dataType: Array.isArray(response.data) ? 'array' : typeof response.data
        });

        const messages = response.data?.data || response.data || [];
        console.log('\n  Raw message samples:');
        messages.slice(0, 2).forEach((msg, index) => {
          console.log(`  Message ${index + 1} raw data:`, {
            ...msg,
            content: msg.content?.substring(0, 50) + '...'
          });
        });

      } catch (error) {
        console.error('  ❌ API request failed:', error);
      }
    }

    console.log('\n4️⃣ Message Sender Resolution');

    // Try to resolve senders
    const unresolvedSenders = new Set();
    const resolvedSenders = new Set();

    chatStore.messages.forEach(msg => {
      if (msg.sender && msg.sender.fullname) {
        resolvedSenders.add(msg.sender_id);
      } else if (msg.sender_id) {
        unresolvedSenders.add(msg.sender_id);
      }
    });

    console.log('  - Messages with resolved senders:', resolvedSenders.size);
    console.log('  - Messages with unresolved senders:', unresolvedSenders.size);
    console.log('  - Unresolved sender IDs:', Array.from(unresolvedSenders));

    console.log('\n5️⃣ Suggested Fixes');

    if (unresolvedSenders.size > 0) {
      console.log('  🔧 Run: window.fixMessageSenders()');
      console.log('  🔧 This will attempt to populate sender data from user store');
    }

    if (userStore.workspaceUsers.length === 0) {
      console.log('  ⚠️ No workspace users loaded!');
      console.log('  🔧 Run: window.loadWorkspaceUsers()');
    }

  } catch (error) {
    console.error('❌ Diagnostic failed:', error);
  }

  console.groupEnd();
}

// Fix message senders by looking up in user store
async function fixMessageSenders() {
  console.log('🔧 Attempting to fix message senders...');

  try {
    const { useChatStore } = await import('@/stores/chat');
    const { useUserStore } = await import('@/stores/user');
    const { useWorkspaceStore } = await import('@/stores/workspace');
    const chatStore = useChatStore();
    const userStore = useUserStore();
    const workspaceStore = useWorkspaceStore();

    // Ensure users are loaded
    if (userStore.workspaceUsers.length === 0) {
      console.log('📥 Loading workspace users first...');
      await workspaceStore.fetchWorkspaceUsers();
      console.log('✅ Loaded', userStore.workspaceUsers.length, 'users');
    }

    // Create user lookup map
    const userMap = new Map();
    userStore.workspaceUsers.forEach(user => {
      userMap.set(user.id, user);
    });

    // Fix messages
    let fixedCount = 0;
    chatStore.messages.forEach(msg => {
      if (msg.sender_id && (!msg.sender || !msg.sender.fullname)) {
        const user = userMap.get(msg.sender_id);
        if (user) {
          msg.sender = {
            id: user.id,
            fullname: user.fullname || user.name || user.email?.split('@')[0],
            email: user.email,
            avatar_url: user.avatar_url || null
          };
          fixedCount++;
        }
      }
    });

    console.log(`✅ Fixed ${fixedCount} messages with missing sender data`);

    // Trigger reactivity update
    chatStore.messages = [...chatStore.messages];

  } catch (error) {
    console.error('❌ Failed to fix senders:', error);
  }
}

// Load workspace users
async function loadWorkspaceUsers() {
  try {
    const { useWorkspaceStore } = await import('@/stores/workspace');
    const workspaceStore = useWorkspaceStore();

    console.log('📥 Loading workspace users...');
    const users = await workspaceStore.fetchWorkspaceUsers();
    console.log('✅ Loaded', users.length, 'users');

    return users;
  } catch (error) {
    console.error('❌ Failed to load users:', error);
  }
}

// Export functions
if (typeof window !== 'undefined') {
  window.diagnoseMessageUserProfiles = diagnoseMessageUserProfiles;
  window.fixMessageSenders = fixMessageSenders;
  window.loadWorkspaceUsers = loadWorkspaceUsers;

  console.log('👤 Message user profile diagnostic loaded:');
  console.log('  - window.diagnoseMessageUserProfiles() - Run diagnostic');
  console.log('  - window.fixMessageSenders() - Fix missing sender data');
  console.log('  - window.loadWorkspaceUsers() - Load workspace users');
}

export { diagnoseMessageUserProfiles, fixMessageSenders, loadWorkspaceUsers }; 