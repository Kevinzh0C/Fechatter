<template>
  <div class="slack-welcome">
    <div class="slack-welcome-content">
      <!-- Welcome Header with User Info -->
      <div class="slack-welcome-header">
        <div class="welcome-avatar">
          {{ userInitials }}
        </div>
        <h1>Welcome back, {{ userName }}!</h1>
        <p>You're in the <strong>{{ workspaceName }}</strong> workspace. Ready to collaborate?</p>
      </div>

      <!-- Quick Stats -->
      <div class="slack-quick-stats">
        <div class="stat-card">
          <div class="stat-number">{{ channelCount }}</div>
          <div class="stat-label">Channels</div>
        </div>
        <div class="stat-card">
          <div class="stat-number">{{ memberCount }}</div>
          <div class="stat-label">Members</div>
        </div>
        <div class="stat-card">
          <div class="stat-number">{{ messageCount }}</div>
          <div class="stat-label">Messages</div>
        </div>
      </div>

      <!-- Quick Actions -->
      <div class="slack-welcome-actions">
        <button @click="$emit('create-channel')" class="slack-btn slack-btn-primary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
          </svg>
          Create a channel
        </button>

        <button @click="$emit('create-dm')" class="slack-btn slack-btn-secondary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z" />
          </svg>
          Send a message
        </button>

        <button @click="$emit('join-channel')" class="slack-btn slack-btn-outline">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
          </svg>
          Browse channels
        </button>
      </div>

      <!-- Recent Activity -->
      <div v-if="recentChannels.length > 0" class="slack-recent-activity">
        <h3>Your channels</h3>
        <div class="channel-list">
          <div v-for="channel in recentChannels" :key="channel.id" @click="$emit('navigate-channel', channel.id)"
            class="channel-card">
            <div class="channel-icon">
              {{ channel.chat_type === 'PrivateChannel' ? '🔒' : '#' }}
            </div>
            <div class="channel-info">
              <div class="channel-name">{{ channel.name }}</div>
              <div class="channel-description">{{ channel.description || 'No description' }}</div>
            </div>
            <div class="channel-arrow">→</div>
          </div>
        </div>
      </div>

      <!-- Feature Highlights -->
      <div class="slack-welcome-features">
        <h3>What you can do here</h3>
        <div class="feature-grid">
          <div class="slack-feature-card">
            <div class="slack-feature-icon">💬</div>
            <h4>Real-time Chat</h4>
            <p>Instant messaging with your team members across channels</p>
          </div>
          <div class="slack-feature-card">
            <div class="slack-feature-icon">📁</div>
            <h4>File Sharing</h4>
            <p>Share documents, images, and files with your colleagues</p>
          </div>
          <div class="slack-feature-card">
            <div class="slack-feature-icon">🔍</div>
            <h4>Smart Search</h4>
            <p>Find any message, file, or conversation instantly with powerful search</p>
          </div>
          <div class="slack-feature-card">
            <div class="slack-feature-icon">👥</div>
            <h4>Team Collaboration</h4>
            <p>Work together efficiently with organized channels and threads</p>
          </div>
        </div>
      </div>

      <!-- Getting Started Tips -->
      <div class="slack-getting-started">
        <h3>Getting started</h3>
        <div class="tip-list">
          <div class="tip-item">
            <span class="tip-number">1</span>
            <span class="tip-text">Join existing channels or create new ones to organize conversations</span>
          </div>
          <div class="tip-item">
            <span class="tip-number">2</span>
            <span class="tip-text">Use @ mentions to notify specific team members</span>
          </div>
          <div class="tip-item">
            <span class="tip-number">3</span>
            <span class="tip-text">Share files by dragging and dropping them into the message area</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { useChatStore } from '@/stores/chat';

// Emits
defineEmits(['create-channel', 'create-dm', 'join-channel', 'navigate-channel']);

// Stores
const authStore = useAuthStore();
const chatStore = useChatStore();

// Computed properties
const userName = computed(() => {
  const user = authStore.user;
  return user?.fullname?.split(' ')[0] || user?.email?.split('@')[0] || 'User';
});

const userInitials = computed(() => {
  const user = authStore.user;
  const name = user?.fullname || user?.email || 'U';
  return name.split(' ').map(n => n.charAt(0)).join('').toUpperCase().slice(0, 2);
});

const workspaceName = computed(() => {
  return authStore.user?.workspace?.name || 'Fechatter Workspace';
});

const channelCount = computed(() => {
  return chatStore.chats?.length || 0;
});

const memberCount = computed(() => {
  // TODO: Implement workspace member count
  return 1;
});

const messageCount = computed(() => {
  // TODO: Implement total message count
  return 0;
});

const recentChannels = computed(() => {
  return chatStore.chats?.slice(0, 3) || [];
});
</script>

<style scoped>
.slack-welcome {
  /* Enable vertical scroll for overflow */
  overflow-y: auto;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 2rem;
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
}

.slack-welcome-content {
  /* Prevent content overflow and allow scroll */
  max-height: calc(100vh - 4rem);
  overflow-y: auto;
  max-width: 800px;
  width: 100%;
  background: white;
  border-radius: 12px;
  padding: 3rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
}

/* Welcome Header */
.slack-welcome-header {
  text-align: center;
  margin-bottom: 2.5rem;
}

.welcome-avatar {
  width: 80px;
  height: 80px;
  border-radius: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  font-weight: 700;
  margin: 0 auto 1rem;
}

.slack-welcome-header h1 {
  font-size: 2rem;
  font-weight: 700;
  color: #1d1c1d;
  margin: 0 0 0.5rem 0;
}

.slack-welcome-header p {
  font-size: 1.1rem;
  color: #616061;
  margin: 0;
}

/* Quick Stats */
.slack-quick-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
  margin-bottom: 2.5rem;
}

.stat-card {
  text-align: center;
  padding: 1.5rem;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.stat-number {
  font-size: 2rem;
  font-weight: 700;
  color: #4a154b;
  margin-bottom: 0.25rem;
}

.stat-label {
  font-size: 0.875rem;
  color: #616061;
  font-weight: 500;
}

/* Actions */
.slack-welcome-actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  margin-bottom: 2.5rem;
  flex-wrap: wrap;
}

.slack-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  font-size: 0.875rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
  text-decoration: none;
}

.slack-btn-primary {
  background: #4a154b;
  color: white;
}

.slack-btn-primary:hover {
  background: #3f0f40;
}

.slack-btn-secondary {
  background: #007a5a;
  color: white;
}

.slack-btn-secondary:hover {
  background: #006749;
}

.slack-btn-outline {
  background: transparent;
  color: #4a154b;
  border: 1px solid #4a154b;
}

.slack-btn-outline:hover {
  background: #4a154b;
  color: white;
}

/* Recent Activity */
.slack-recent-activity {
  margin-bottom: 2.5rem;
}

.slack-recent-activity h3 {
  font-size: 1.25rem;
  font-weight: 600;
  color: #1d1c1d;
  margin: 0 0 1rem 0;
}

.channel-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.channel-card {
  display: flex;
  align-items: center;
  padding: 1rem;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e9ecef;
  cursor: pointer;
  transition: all 0.2s;
}

.channel-card:hover {
  background: #e9ecef;
  border-color: #4a154b;
}

.channel-icon {
  font-size: 1.25rem;
  margin-right: 0.75rem;
}

.channel-info {
  flex: 1;
}

.channel-name {
  font-weight: 600;
  color: #1d1c1d;
  margin-bottom: 0.25rem;
}

.channel-description {
  font-size: 0.875rem;
  color: #616061;
}

.channel-arrow {
  color: #616061;
  font-size: 1.25rem;
}

/* Features */
.slack-welcome-features {
  margin-bottom: 2.5rem;
}

.slack-welcome-features h3 {
  font-size: 1.25rem;
  font-weight: 600;
  color: #1d1c1d;
  margin: 0 0 1.5rem 0;
  text-align: center;
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1.5rem;
}

.slack-feature-card {
  text-align: center;
  padding: 1.5rem;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.slack-feature-icon {
  font-size: 2rem;
  margin-bottom: 0.75rem;
}

.slack-feature-card h4 {
  font-size: 1rem;
  font-weight: 600;
  color: #1d1c1d;
  margin: 0 0 0.5rem 0;
}

.slack-feature-card p {
  font-size: 0.875rem;
  color: #616061;
  margin: 0;
  line-height: 1.4;
}

/* Getting Started */
.slack-getting-started h3 {
  font-size: 1.25rem;
  font-weight: 600;
  color: #1d1c1d;
  margin: 0 0 1rem 0;
}

.tip-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.tip-item {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.tip-number {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: #4a154b;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
  font-weight: 600;
  flex-shrink: 0;
}

.tip-text {
  font-size: 0.875rem;
  color: #616061;
  line-height: 1.4;
  margin-top: 0.125rem;
}

/* Responsive */
@media (max-width: 768px) {
  .slack-welcome-content {
    padding: 2rem 1.5rem;
  }

  .slack-quick-stats {
    grid-template-columns: 1fr;
  }

  .slack-welcome-actions {
    flex-direction: column;
  }

  .feature-grid {
    grid-template-columns: 1fr;
  }
}
</style>