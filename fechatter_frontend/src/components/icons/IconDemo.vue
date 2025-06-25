<template>
  <div :class="{ 'dark-mode': isDarkMode, 'demo-container': true }">
    <h1>Fechatter Vue Icon Components</h1>
    
    <div class="controls">
      <div class="control-group">
        <button @click="toggleDarkMode">{{ isDarkMode ? 'Light Mode' : 'Dark Mode' }}</button>
        <button @click="increaseSize">Size +</button>
        <button @click="decreaseSize">Size -</button>
        <button @click="toggleGradient">{{ preserveGradient ? 'Custom Colors' : 'Default Gradient' }}</button>
      </div>
      
      <div v-if="!preserveGradient" class="color-inputs">
        <div>
          <label for="startColor">Start Color:</label>
          <input type="color" id="startColor" v-model="startColor">
        </div>
        <div>
          <label for="endColor">End Color:</label>
          <input type="color" id="endColor" v-model="endColor">
        </div>
      </div>
    </div>
    
    <div class="icon-grid">
      <div class="icon-item">
        <LogoBubbleNetwork 
          :size="size" 
          :preserveGradient="preserveGradient"
          :startColor="startColor"
          :endColor="endColor"
          :clickable="true"
          title="Logo Bubble Network"
          @click="handleClick('LogoBubbleNetwork')"
        />
        <p>LogoBubbleNetwork</p>
      </div>
      
      <div class="icon-item">
        <ChatAiOutline 
          :size="size" 
          :preserveGradient="preserveGradient"
          :startColor="startColor"
          :endColor="endColor"
          :clickable="true"
          title="Chat AI Outline"
          @click="handleClick('ChatAiOutline')"
        />
        <p>ChatAiOutline</p>
      </div>
      
      <div class="icon-item">
        <ChatAiDark 
          :size="size" 
          :preserveGradient="preserveGradient"
          :startColor="startColor"
          :endColor="endColor"
          :clickable="true"
          title="Chat AI Dark"
          @click="handleClick('ChatAiDark')"
        />
        <p>ChatAiDark</p>
      </div>
      
      <div class="icon-item">
        <MessageBubble 
          :size="size" 
          :preserveGradient="preserveGradient"
          :startColor="startColor"
          :endColor="endColor"
          :clickable="true"
          title="Message Bubble"
          @click="handleClick('MessageBubble')"
        />
        <p>MessageBubble</p>
      </div>
    </div>
    
    <div class="api-demo">
      <h2>Current Props</h2>
      <pre>{{ JSON.stringify({
        size,
        preserveGradient,
        startColor,
        endColor,
        clickable: true,
        isDarkMode
      }, null, 2) }}</pre>
    </div>
  </div>
</template>

<script lang="ts" src="./IconDemo.ts"></script>

<style scoped>
.demo-container {
  font-family: 'Arial', sans-serif;
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
  background-color: #f5f5f5;
  border-radius: 8px;
  transition: background-color 0.3s ease;
}

.dark-mode {
  background-color: #121212;
  color: #ffffff;
}

h1 {
  text-align: center;
  margin-bottom: 30px;
}

.controls {
  margin-bottom: 30px;
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.control-group {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  justify-content: center;
}

button {
  padding: 8px 16px;
  background-color: #3949AB;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

button:hover {
  background-color: #303F9F;
}

.dark-mode button {
  background-color: #00BCD4;
}

.dark-mode button:hover {
  background-color: #00ACC1;
}

.color-inputs {
  display: flex;
  gap: 20px;
  justify-content: center;
  margin-top: 10px;
}

.color-inputs label {
  margin-right: 8px;
}

.icon-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 30px;
  margin-bottom: 30px;
}

.icon-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.api-demo {
  background-color: rgba(0, 0, 0, 0.05);
  padding: 15px;
  border-radius: 4px;
}

.dark-mode .api-demo {
  background-color: rgba(255, 255, 255, 0.05);
}

pre {
  overflow-x: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
}
</style>
