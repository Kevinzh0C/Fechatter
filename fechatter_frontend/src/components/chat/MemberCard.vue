<!--
  MemberCard.vue
  Jobs-Inspired Member Card Component
  
  Philosophy:
  - Humanize: Natural, breathing interactions
  - Focus: Essential member info at a glance
  - Anticipate: Quick actions on hover
  - Perfect: Pixel-perfect alignment and spacing
-->
<template>
  <div ref="cardRef" :class="cardClasses" @click="handleClick" @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave" @touchstart="handleTouchStart" @touchend="handleTouchEnd">
    <!-- Member Avatar with Status -->
    <div class="member-avatar-container">
      <div class="member-avatar" :class="avatarClasses">
        <img v-if="member.avatar_url" :src="member.avatar_url" :alt="member.fullname" class="avatar-image"
          @error="handleAvatarError" />
        <div v-else class="avatar-initials">
          {{ getInitials(member.fullname) }}
        </div>

        <!-- Status Indicator -->
        <div :class="statusClasses" :title="statusTitle"></div>
      </div>
    </div>

    <!-- Member Info -->
    <div class="member-info">
      <div class="member-name-container">
        <span class="member-name">{{ member.fullname }}</span>
        <span v-if="isCurrentUser" class="current-user-badge">You</span>
        <span v-if="member.role && member.role !== 'member'" class="member-role">
          {{ formatRole(member.role) }}
        </span>
      </div>

      <div class="member-details">
        <span class="member-email">{{ member.email }}</span>
        <span v-if="lastSeen" class="member-last-seen">{{ lastSeen }}</span>
      </div>
    </div>

    <!-- Quick Actions - Appear on hover -->
    <Transition name="actions-fade">
      <div v-if="showActions" class="member-actions">

        <!-- Message Action -->
        <ActionButton icon="message-circle" tooltip="Send message" variant="default" size="small"
          @click.stop="handleStartDM" />

        <!-- Profile Action -->
        <ActionButton icon="user" tooltip="View profile" variant="default" size="small"
          @click.stop="handleViewProfile" />

        <!-- More Actions -->
        <ContextualMenu v-if="hasMoreActions" v-model:open="showMoreMenu" :items="moreActionItems"
          @action="handleMoreAction">
          <ActionButton icon="more-horizontal" tooltip="More actions" variant="default" size="small"
            :active="showMoreMenu" @click.stop />
        </ContextualMenu>
      </div>
    </Transition>

    <!-- Ripple Effect -->
    <div v-if="showRipple" ref="rippleRef" class="member-ripple" :style="rippleStyle"></div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import ActionButton from './ActionButton.vue';
import ContextualMenu from './ContextualMenu.vue';

// Props
const props = defineProps({
  member: {
    type: Object,
    required: true
  },
  status: {
    type: String,
    default: 'offline',
    validator: (value) => ['online', 'away', 'busy', 'offline'].includes(value)
  },
  isCurrentUser: {
    type: Boolean,
    default: false
  },
  canManage: {
    type: Boolean,
    default: false
  }
});

// Emits
const emit = defineEmits([
  'click',
  'start-dm',
  'view-profile',
  'manage-member',
  'remove-member'
]);

// Reactive state
const cardRef = ref(null);
const rippleRef = ref(null);
const isHovered = ref(false);
const isPressed = ref(false);
const showRipple = ref(false);
const showMoreMenu = ref(false);
const rippleStyle = ref({});
const avatarError = ref(false);

// Computed properties
const cardClasses = computed(() => [
  'member-card',
  {
    'member-card--hovered': isHovered.value,
    'member-card--pressed': isPressed.value,
    'member-card--current-user': props.isCurrentUser,
    'member-card--online': props.status === 'online',
    'member-card--clickable': true
  }
]);

const avatarClasses = computed(() => [
  {
    'member-avatar--online': props.status === 'online',
    'member-avatar--away': props.status === 'away',
    'member-avatar--busy': props.status === 'busy',
    'member-avatar--current-user': props.isCurrentUser
  }
]);

const statusClasses = computed(() => [
  'status-indicator',
  `status-indicator--${props.status}`
]);

const statusTitle = computed(() => {
  const statusMap = {
    online: 'Online',
    away: 'Away',
    busy: 'Busy - Do not disturb',
    offline: 'Offline'
  };
  return statusMap[props.status] || 'Unknown status';
});

const showActions = computed(() =>
  isHovered.value && !props.isCurrentUser
);

const hasMoreActions = computed(() =>
  props.canManage || moreActionItems.value.length > 0
);

const moreActionItems = computed(() => {
  const items = [];

  if (props.canManage) {
    items.push(
      {
        id: 'manage',
        label: 'Manage Member',
        icon: 'shield',
        action: () => handleManageMember()
      },
      { type: 'divider' },
      {
        id: 'remove',
        label: 'Remove from Chat',
        icon: 'user-minus',
        variant: 'danger',
        action: () => handleRemoveMember()
      }
    );
  }

  return items;
});

const lastSeen = computed(() => {
  if (props.status === 'online') return 'Active now';
  if (!props.member.last_seen) return null;

  try {
    const lastSeenDate = new Date(props.member.last_seen);
    const now = new Date();
    const diffMs = now - lastSeenDate;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffDays > 0) {
      return `Last seen ${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
    } else if (diffHours > 0) {
      return `Last seen ${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
    } else if (diffMins > 0) {
      return `Last seen ${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
    } else {
      return 'Last seen just now';
    }
  } catch {
    return null;
  }
});

// Event handlers
const handleClick = (event) => {
  createRippleEffect(event);
  emit('click', props.member);
};

const handleMouseEnter = () => {
  isHovered.value = true;
};

const handleMouseLeave = () => {
  isHovered.value = false;
  showMoreMenu.value = false;
};

const handleTouchStart = () => {
  isPressed.value = true;
};

const handleTouchEnd = () => {
  isPressed.value = false;
};

const handleStartDM = () => {
  emit('start-dm', props.member);
};

const handleViewProfile = () => {
  emit('view-profile', props.member);
};

const handleManageMember = () => {
  emit('manage-member', props.member);
};

const handleRemoveMember = () => {
  emit('remove-member', props.member);
};

const handleMoreAction = (action) => {
  action.action();
  showMoreMenu.value = false;
};

const handleAvatarError = () => {
  avatarError.value = true;
};

// Utility functions
const getInitials = (name) => {
  if (!name) return '?';
  return name
    .split(' ')
    .map(word => word.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 2);
};

const formatRole = (role) => {
  const roleMap = {
    admin: 'Admin',
    moderator: 'Mod',
    owner: 'Owner'
  };
  return roleMap[role.toLowerCase()] || role;
};

const createRippleEffect = (event) => {
  if (!cardRef.value) return;

  const card = cardRef.value;
  const rect = card.getBoundingClientRect();
  const size = Math.max(rect.width, rect.height);
  const x = event.clientX - rect.left - size / 2;
  const y = event.clientY - rect.top - size / 2;

  rippleStyle.value = {
    width: `${size}px`,
    height: `${size}px`,
    left: `${x}px`,
    top: `${y}px`,
  };

  showRipple.value = true;

  setTimeout(() => {
    showRipple.value = false;
  }, 600);
};
</script>

<style scoped>
/*
  Jobs-Inspired Member Card Design
  - Clean, minimal aesthetic
  - Smooth, natural animations
  - Perfect spacing and alignment
  - Premium hover effects
*/

.member-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid transparent;
  cursor: pointer;
  overflow: hidden;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  user-select: none;
}

.member-card--clickable:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.08);
  transform: translateY(-1px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
}

.member-card--pressed {
  transform: translateY(0) scale(0.98);
}

.member-card--current-user {
  background: linear-gradient(135deg, rgba(0, 122, 255, 0.08), rgba(88, 86, 214, 0.08));
  border-color: rgba(0, 122, 255, 0.2);
}

.member-card--current-user:hover {
  background: linear-gradient(135deg, rgba(0, 122, 255, 0.12), rgba(88, 86, 214, 0.12));
  border-color: rgba(0, 122, 255, 0.3);
}

/* Avatar Container */
.member-avatar-container {
  position: relative;
  flex-shrink: 0;
}

.member-avatar {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  overflow: hidden;
  transition: all 0.2s ease;
  position: relative;
  border: 2px solid rgba(255, 255, 255, 0.1);
}

.member-avatar--current-user {
  border-color: rgba(0, 122, 255, 0.3);
  box-shadow: 0 0 12px rgba(0, 122, 255, 0.2);
}

.member-avatar--online {
  border-color: rgba(35, 165, 90, 0.3);
}

.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 10px;
}

.avatar-initials {
  font-size: 16px;
  font-weight: 600;
  color: white;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}

/* Status Indicator */
.status-indicator {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 2px solid rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
}

.status-indicator--online {
  background: linear-gradient(135deg, #23a55a, #16a34a);
  box-shadow: 0 0 8px rgba(35, 165, 90, 0.4);
}

.status-indicator--away {
  background: linear-gradient(135deg, #f59e0b, #eab308);
  box-shadow: 0 0 8px rgba(245, 158, 11, 0.4);
}

.status-indicator--busy {
  background: linear-gradient(135deg, #ef4444, #dc2626);
  box-shadow: 0 0 8px rgba(239, 68, 68, 0.4);
}

.status-indicator--offline {
  background: #6b7280;
}

/* Member Info */
.member-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.member-name-container {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.member-name {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.current-user-badge {
  padding: 2px 6px;
  background: linear-gradient(135deg, #007AFF, #5856D6);
  color: white;
  font-size: 10px;
  font-weight: 600;
  border-radius: 6px;
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.member-role {
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
  font-size: 10px;
  font-weight: 500;
  border-radius: 6px;
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.member-details {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.member-email {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.member-last-seen {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  font-weight: 400;
}

/* Quick Actions */
.member-actions {
  display: flex;
  gap: 4px;
  margin-left: auto;
  padding-left: 8px;
}

.actions-fade-enter-active,
.actions-fade-leave-active {
  transition: all 0.2s ease;
}

.actions-fade-enter-from,
.actions-fade-leave-to {
  opacity: 0;
  transform: translateX(8px);
}

/* Ripple Effect */
.member-ripple {
  position: absolute;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.15);
  pointer-events: none;
  transform: scale(0);
  animation: memberRipple 0.6s linear;
}

@keyframes memberRipple {
  to {
    transform: scale(2);
    opacity: 0;
  }
}

/* Dark mode adaptations */
@media (prefers-color-scheme: dark) {
  .member-card {
    background: rgba(255, 255, 255, 0.03);
  }

  .member-card--clickable:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .member-name {
    color: #ffffff;
  }

  .member-email {
    color: rgba(255, 255, 255, 0.7);
  }
}

/* Responsive design */
@media (max-width: 480px) {
  .member-card {
    padding: 10px;
    gap: 10px;
  }

  .member-avatar {
    width: 36px;
    height: 36px;
  }

  .avatar-initials {
    font-size: 14px;
  }

  .status-indicator {
    width: 12px;
    height: 12px;
  }
}

/* High contrast mode */
@media (prefers-contrast: high) {
  .member-card {
    border: 1px solid rgba(255, 255, 255, 0.3);
  }

  .member-avatar {
    border-width: 3px;
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {

  .member-card,
  .member-avatar,
  .member-actions,
  .member-ripple {
    transition: none;
    animation: none;
  }
}
</style>