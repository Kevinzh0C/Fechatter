// Duplicate message debugger
import { useChatStore } from '@/stores/chat';

// Check for duplicate messages in the data layer
window.debugDuplicates = function () {
  const chatStore = useChatStore();
  const messages = chatStore._rawMessages;
  const seen = new Map();
  const duplicates = [];

  messages.forEach((msg, index) => {
    const key = msg.id || msg.temp_id || msg.idempotency_key;
    if (seen.has(key)) {
      duplicates.push({
        index1: seen.get(key),
        index2: index,
        key: key,
        message1: messages[seen.get(key)],
        message2: msg
      });
    } else {
      seen.set(key, index);
    }
  });

  if (duplicates.length > 0) {
    if (import.meta.env.DEV) {
      console.log('🔴 Found duplicates in data:', duplicates);
    }
    return duplicates;
  } else {
    if (import.meta.env.DEV) {
      console.log('✅ No duplicates found in data');
    }
    return [];
  }
};

// Check for duplicate messages in the DOM
window.debugVirtualList = function () {
  const virtualItems = document.querySelectorAll('[data-virtual-item="true"]');
  const seen = new Map();
  const duplicates = [];

  virtualItems.forEach((el, index) => {
    const messageId = el.getAttribute('data-message-id');
    if (seen.has(messageId)) {
      duplicates.push({
        messageId,
        index1: seen.get(messageId),
        index2: index,
        element1: virtualItems[seen.get(messageId)],
        element2: el
      });
    } else {
      seen.set(messageId, index);
    }
  });

  if (duplicates.length > 0) {
    if (import.meta.env.DEV) {
      console.log('🔴 Found duplicate DOM elements:', duplicates);
    }
    return duplicates;
  } else {
    if (import.meta.env.DEV) {
      console.log('✅ No duplicate DOM elements');
    }
    return [];
  }
};

// Check for duplicate messages by content and sender
window.debugDuplicateContent = function () {
  const chatStore = useChatStore();
  const messages = chatStore._rawMessages;
  const contentMap = new Map();
  const duplicates = [];

  messages.forEach((msg, index) => {
    const contentKey = `${msg.sender_id}-${msg.content}-${Math.floor(new Date(msg.created_at).getTime() / 1000)}`;
    if (contentMap.has(contentKey)) {
      duplicates.push({
        index1: contentMap.get(contentKey),
        index2: index,
        contentKey: contentKey,
        message1: messages[contentMap.get(contentKey)],
        message2: msg
      });
    } else {
      contentMap.set(contentKey, index);
    }
  });

  if (duplicates.length > 0) {
    if (import.meta.env.DEV) {
      console.log('🔴 Found duplicate content:', duplicates);
    }
    return duplicates;
  } else {
    if (import.meta.env.DEV) {
      console.log('✅ No duplicate content found');
    }
    return [];
  }
};

// Debug all duplicate issues
window.debugAllDuplicates = function () {
  if (import.meta.env.DEV) {
    console.log('=== Checking for duplicates ===');
  }

  const dataResults = window.debugDuplicates();
  const domResults = window.debugVirtualList();
  const contentResults = window.debugDuplicateContent();

  if (import.meta.env.DEV) {
    console.log('=== Summary ===');
    console.log(`Data duplicates: ${dataResults.length}`);
    console.log(`DOM duplicates: ${domResults.length}`);
    console.log(`Content duplicates: ${contentResults.length}`);
  }

  return {
    data: dataResults,
    dom: domResults,
    content: contentResults
  };
};

if (import.meta.env.DEV) {
  console.log('💡 Duplicate debugger loaded. Use window.debugAllDuplicates() to check for duplicates');
}