<template>
  <div class="search-page">
    <div class="search-header">
      <h1>Search Messages</h1>
      <p>Find any message, file, or conversation in your workspace</p>
    </div>
    
    <div class="search-form">
      <div class="search-input-container">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" class="search-icon">
          <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
        </svg>
        <input 
          v-model="searchQuery" 
          type="text" 
          placeholder="Search for messages, files, channels..."
          class="search-input"
          @keyup.enter="performSearch"
        >
        <button 
          @click="performSearch" 
          :disabled="!searchQuery.trim()"
          class="search-button"
        >
          Search
        </button>
      </div>
    </div>
    
    <div v-if="searchResults.length > 0" class="search-results">
      <h2>Results</h2>
      <div class="results-list">
        <!-- Search results would go here -->
        <p class="no-results">Search results will be displayed here</p>
      </div>
    </div>
    
    <div v-else-if="hasSearched" class="no-results">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" class="no-results-icon">
        <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
      </svg>
      <h3>No results found</h3>
      <p>Try adjusting your search terms or check for typos</p>
    </div>
    
    <div v-else class="search-tips">
      <h2>Search Tips</h2>
      <div class="tips-grid">
        <div class="tip-card">
          <h3>Basic Search</h3>
          <p>Just type your keywords to search across all messages</p>
        </div>
        <div class="tip-card">
          <h3>📅 By Date</h3>
          <p>Use "after:2024-01-01" or "before:yesterday"</p>
        </div>
        <div class="tip-card">
          <h3>USER: By Person</h3>
          <p>Use "from:@username" to find messages from someone</p>
        </div>
        <div class="tip-card">
          <h3>#️⃣ In Channel</h3>
          <p>Use "in:#channel-name" to search within a channel</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';

// Reactive state
const searchQuery = ref('');
const searchResults = ref([]);
const hasSearched = ref(false);

// Methods
function performSearch() {
  if (!searchQuery.value.trim()) return;
  
  hasSearched.value = true;
  // TODO: Implement actual search functionality
  console.log('Searching for:', searchQuery.value);
  
  // Mock search - replace with actual API call
  searchResults.value = [];
}
</script>

<style scoped>
.search-page {
  padding: 40px;
  max-width: 800px;
  margin: 0 auto;
  height: 100%;
  overflow-y: auto;
}

.search-header {
  text-align: center;
  margin-bottom: 40px;
}

.search-header h1 {
  font-size: 32px;
  font-weight: 700;
  color: #1d1c1d;
  margin: 0 0 12px 0;
}

.search-header p {
  font-size: 16px;
  color: #616061;
  margin: 0;
}

.search-form {
  margin-bottom: 40px;
}

.search-input-container {
  position: relative;
  display: flex;
  align-items: center;
  background: white;
  border: 2px solid #d1d9e0;
  border-radius: 12px;
  padding: 16px;
  transition: border-color 0.2s;
}

.search-input-container:focus-within {
  border-color: #4a154b;
}

.search-icon {
  color: #616061;
  margin-right: 12px;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 16px;
  color: #1d1c1d;
  background: transparent;
}

.search-input::placeholder {
  color: #9b9b9b;
}

.search-button {
  background: #4a154b;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  margin-left: 12px;
  transition: background 0.2s;
}

.search-button:hover:not(:disabled) {
  background: #3f0f40;
}

.search-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.search-results h2 {
  font-size: 20px;
  font-weight: 600;
  color: #1d1c1d;
  margin: 0 0 20px 0;
}

.no-results {
  text-align: center;
  padding: 60px 20px;
  color: #616061;
}

.no-results-icon {
  color: #d1d9e0;
  margin-bottom: 16px;
}

.no-results h3 {
  font-size: 18px;
  font-weight: 600;
  color: #1d1c1d;
  margin: 0 0 8px 0;
}

.no-results p {
  font-size: 14px;
  margin: 0;
}

.search-tips {
  margin-top: 40px;
}

.search-tips h2 {
  font-size: 24px;
  font-weight: 700;
  color: #1d1c1d;
  margin: 0 0 24px 0;
  text-align: center;
}

.tips-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
}

.tip-card {
  background: #f8f9fa;
  padding: 20px;
  border-radius: 8px;
  text-align: center;
}

.tip-card h3 {
  font-size: 16px;
  font-weight: 600;
  color: #1d1c1d;
  margin: 0 0 8px 0;
}

.tip-card p {
  font-size: 14px;
  color: #616061;
  margin: 0;
}
</style> 