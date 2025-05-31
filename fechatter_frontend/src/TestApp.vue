<template>
  <div class="test-app">
    <h1>Fechatter Test App</h1>
    <p>If you can see this, Vue is working!</p>
    <div class="status">
      <p>Current Time: {{ currentTime }}</p>
      <p>Message Count: {{ messages.length }}</p>
      <button @click="addMessage">Add Test Message</button>
    </div>
    <div class="messages">
      <div v-for="msg in messages" :key="msg.id" class="message">
        {{ msg.text }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';

const messages = ref([
  { id: 1, text: 'Test message 1' },
  { id: 2, text: 'Test message 2' }
]);

const currentTime = ref(new Date().toLocaleTimeString());

function addMessage() {
  messages.value.push({
    id: Date.now(),
    text: `New message at ${new Date().toLocaleTimeString()}`
  });
}

// Update time every second
setInterval(() => {
  currentTime.value = new Date().toLocaleTimeString();
}, 1000);
</script>

<style scoped>
.test-app {
  padding: 20px;
  font-family: Arial, sans-serif;
}

.status {
  background: #f0f0f0;
  padding: 10px;
  margin: 20px 0;
  border-radius: 5px;
}

button {
  background: #007bff;
  color: white;
  border: none;
  padding: 5px 15px;
  border-radius: 3px;
  cursor: pointer;
}

button:hover {
  background: #0056b3;
}

.messages {
  margin-top: 20px;
}

.message {
  padding: 10px;
  margin: 5px 0;
  background: #e9ecef;
  border-radius: 3px;
}
</style> 