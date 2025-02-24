<template>
  <div class="perfect-search-demo">
    <!-- Header -->
    <header class="demo-header">
      <div class="header-content">
        <h1 class="demo-title">Perfect Search</h1>
        <p class="demo-subtitle">
          Search reimagined with Jobs-inspired design principles
        </p>
        
        <div class="design-principles">
          <div class="principle">
            <div class="principle-icon">⚡</div>
            <span>Instantly responsive</span>
          </div>
          <div class="principle">
            <div class="principle-icon">🎯</div>
            <span>Intelligently focused</span>
          </div>
          <div class="principle">
            <div class="principle-icon">✨</div>
            <span>Elegantly simple</span>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Demo Area -->
    <main class="demo-main">
      <div class="demo-container">
        
        <!-- Perfect Search Integration -->
        <div class="search-demo-section">
          <h2 class="section-title">Try the Perfect Search</h2>
          <p class="section-description">
            Experience search the way it should be - instant, intelligent, and intuitive.
          </p>
          
          <div class="search-showcase">
            <PerfectSearchIntegration />
          </div>
        </div>

        <!-- Features Grid -->
        <div class="features-section">
          <h2 class="section-title">Design Philosophy</h2>
          
          <div class="features-grid">
            <div class="feature-card">
              <div class="feature-header">
                <div class="feature-icon minimalist">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <circle cx="12" cy="12" r="10"/>
                    <path d="M12 6v6l4 2"/>
                  </svg>
                </div>
                <h3 class="feature-title">Instant Response</h3>
              </div>
              <p class="feature-description">
                Search results appear as you type, with intelligent debouncing 
                and caching for lightning-fast performance.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <div class="feature-icon semantic">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <path d="M9 12l2 2 4-4"/>
                    <path d="M21 12c0 4.97-4.03 9-9 9s-9-4.03-9-9 4.03-9 9-9 9 4.03 9 9z"/>
                  </svg>
                </div>
                <h3 class="feature-title">Smart Strategy</h3>
              </div>
              <p class="feature-description">
                Multiple search strategies - semantic, fuzzy, exact match - 
                automatically selected based on your query.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <div class="feature-icon elegant">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                  </svg>
                </div>
                <h3 class="feature-title">Elegant Interface</h3>
              </div>
              <p class="feature-description">
                Clean typography, perfect spacing, and smooth animations 
                create a delightful search experience.
              </p>
            </div>

            <div class="feature-card">
              <div class="feature-header">
                <div class="feature-icon accessible">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                    <circle cx="9" cy="7" r="4"/>
                    <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
                    <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
                  </svg>
                </div>
                <h3 class="feature-title">Universal Access</h3>
              </div>
              <p class="feature-description">
                Keyboard shortcuts, screen reader support, and high contrast 
                modes ensure everyone can search effectively.
              </p>
            </div>
          </div>
        </div>

        <!-- Search Statistics -->
        <div class="stats-section" v-if="searchStats">
          <h2 class="section-title">Performance Metrics</h2>
          
          <div class="stats-grid">
            <div class="stat-card">
              <div class="stat-value">{{ searchStats.totalSearches }}</div>
              <div class="stat-label">Total Searches</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">{{ Math.round(searchStats.avgResponseTime) }}ms</div>
              <div class="stat-label">Avg Response</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">{{ Math.round(searchStats.cacheHitRate * 100) }}%</div>
              <div class="stat-label">Cache Hit Rate</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">{{ searchStats.cacheSize }}</div>
              <div class="stat-label">Cached Results</div>
            </div>
          </div>
        </div>

        <!-- Code Example -->
        <div class="code-section">
          <h2 class="section-title">Implementation</h2>
          <p class="section-description">
            Clean, maintainable code that follows modern Vue.js best practices.
          </p>
          
          <div class="code-example">
            <div class="code-header">
              <span class="code-language">Vue.js</span>
              <button @click="copyCode" class="copy-button">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                  <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                </svg>
                Copy
              </button>
            </div>
            <pre class="code-content"><code>// Perfect Search Integration
import { perfectSearchService } from '@/services/perfectSearchService'

const search = async (query) => {
  const results = await perfectSearchService.search({
    query,
    chatId: currentChat.id,
    strategies: ['semantic', 'fuzzy', 'exact']
  })
  
  return results.hits
}</code></pre>
          </div>
        </div>
      </div>
    </main>

    <!-- Footer -->
    <footer class="demo-footer">
      <div class="footer-content">
        <p class="footer-text">
          Crafted with attention to detail and respect for user experience.
        </p>
        <div class="footer-links">
          <button @click="resetDemo" class="footer-link">Reset Demo</button>
          <button @click="openDocumentation" class="footer-link">Documentation</button>
        </div>
      </div>
    </footer>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import PerfectSearchIntegration from '@/components/search/PerfectSearchIntegration.vue'
import perfectSearchService from '@/services/perfectSearchService'

// State
const searchStats = ref(null)
const updateInterval = ref(null)

// Methods
const loadSearchStats = () => {
  try {
    searchStats.value = perfectSearchService.getMetrics()
  } catch (error) {
    console.warn('Failed to load search stats:', error)
  }
}

const copyCode = async () => {
  const code = `// Perfect Search Integration
import { perfectSearchService } from '@/services/perfectSearchService'

const search = async (query) => {
  const results = await perfectSearchService.search({
    query,
    chatId: currentChat.id,
    strategies: ['semantic', 'fuzzy', 'exact']
  })
  
  return results.hits
}`

  try {
    await navigator.clipboard.writeText(code)
    // You could add a toast notification here
    console.log('Code copied to clipboard!')
  } catch (error) {
    console.error('Failed to copy code:', error)
  }
}

const resetDemo = () => {
  perfectSearchService.clearCache()
  loadSearchStats()
  console.log('Demo reset - cache cleared')
}

const openDocumentation = () => {
  // Open documentation in new tab
  window.open('/docs/search', '_blank')
}

// Lifecycle
onMounted(() => {
  // Load initial stats
  loadSearchStats()
  
  // Update stats every 5 seconds
  updateInterval.value = setInterval(loadSearchStats, 5000)
})

onUnmounted(() => {
  if (updateInterval.value) {
    clearInterval(updateInterval.value)
  }
})
</script>

<style scoped>
/* 🎨 Perfect Search Demo - Jobs-inspired Design */

/* Design System */
:root {
  --color-demo-bg: #fafafa;
  --color-demo-surface: #ffffff;
  --color-demo-text: #1d1d1f;
  --color-demo-text-secondary: #6e6e73;
  --color-demo-accent: #007aff;
  --color-demo-border: #d2d2d7;
  
  --space-demo-xs: 8px;
  --space-demo-sm: 16px;
  --space-demo-md: 24px;
  --space-demo-lg: 32px;
  --space-demo-xl: 48px;
  --space-demo-2xl: 64px;
  --space-demo-3xl: 96px;
  
  --radius-demo: 12px;
  --radius-demo-lg: 16px;
  
  --shadow-demo-sm: 0 1px 3px rgba(0, 0, 0, 0.1);
  --shadow-demo-md: 0 4px 12px rgba(0, 0, 0, 0.1);
  --shadow-demo-lg: 0 8px 24px rgba(0, 0, 0, 0.12);
}

/* Layout */
.perfect-search-demo {
  min-height: 100vh;
  background: var(--color-demo-bg);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
}

/* Header */
.demo-header {
  background: linear-gradient(135deg, var(--color-demo-surface), #f5f5f7);
  border-bottom: 1px solid var(--color-demo-border);
  padding: var(--space-demo-3xl) var(--space-demo-md) var(--space-demo-2xl);
}

.header-content {
  max-width: 1200px;
  margin: 0 auto;
  text-align: center;
}

.demo-title {
  font-size: 48px;
  font-weight: 700;
  color: var(--color-demo-text);
  margin: 0 0 var(--space-demo-sm);
  letter-spacing: -0.02em;
}

.demo-subtitle {
  font-size: 20px;
  color: var(--color-demo-text-secondary);
  margin: 0 0 var(--space-demo-xl);
  line-height: 1.5;
}

.design-principles {
  display: flex;
  justify-content: center;
  gap: var(--space-demo-xl);
  flex-wrap: wrap;
}

.principle {
  display: flex;
  align-items: center;
  gap: var(--space-demo-xs);
  font-size: 16px;
  color: var(--color-demo-text-secondary);
  font-weight: 500;
}

.principle-icon {
  font-size: 20px;
}

/* Main Content */
.demo-main {
  padding: var(--space-demo-2xl) var(--space-demo-md);
}

.demo-container {
  max-width: 1200px;
  margin: 0 auto;
}

/* Section Styles */
.search-demo-section,
.features-section,
.stats-section,
.code-section {
  margin-bottom: var(--space-demo-3xl);
}

.section-title {
  font-size: 32px;
  font-weight: 600;
  color: var(--color-demo-text);
  margin: 0 0 var(--space-demo-sm);
  text-align: center;
  letter-spacing: -0.01em;
}

.section-description {
  font-size: 18px;
  color: var(--color-demo-text-secondary);
  text-align: center;
  margin: 0 0 var(--space-demo-xl);
  line-height: 1.6;
  max-width: 600px;
  margin-left: auto;
  margin-right: auto;
}

/* Search Showcase */
.search-showcase {
  display: flex;
  justify-content: center;
  padding: var(--space-demo-xl);
  background: var(--color-demo-surface);
  border-radius: var(--radius-demo-lg);
  box-shadow: var(--shadow-demo-md);
}

/* Features Grid */
.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: var(--space-demo-lg);
}

.feature-card {
  background: var(--color-demo-surface);
  border-radius: var(--radius-demo);
  padding: var(--space-demo-lg);
  box-shadow: var(--shadow-demo-sm);
  transition: all 0.2s ease;
}

.feature-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-demo-md);
}

.feature-header {
  display: flex;
  align-items: center;
  gap: var(--space-demo-sm);
  margin-bottom: var(--space-demo-md);
}

.feature-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.feature-icon.minimalist { background: linear-gradient(135deg, #667eea, #764ba2); }
.feature-icon.semantic { background: linear-gradient(135deg, #f093fb, #f5576c); }
.feature-icon.elegant { background: linear-gradient(135deg, #4facfe, #00f2fe); }
.feature-icon.accessible { background: linear-gradient(135deg, #43e97b, #38f9d7); }

.feature-icon svg {
  width: 20px;
  height: 20px;
}

.feature-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-demo-text);
  margin: 0;
}

.feature-description {
  font-size: 15px;
  color: var(--color-demo-text-secondary);
  line-height: 1.6;
  margin: 0;
}

/* Stats Grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: var(--space-demo-md);
}

.stat-card {
  background: var(--color-demo-surface);
  border-radius: var(--radius-demo);
  padding: var(--space-demo-lg);
  text-align: center;
  box-shadow: var(--shadow-demo-sm);
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--color-demo-accent);
  margin-bottom: var(--space-demo-xs);
}

.stat-label {
  font-size: 14px;
  color: var(--color-demo-text-secondary);
  font-weight: 500;
}

/* Code Example */
.code-example {
  background: #1e1e1e;
  border-radius: var(--radius-demo);
  overflow: hidden;
  box-shadow: var(--shadow-demo-md);
}

.code-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-demo-sm) var(--space-demo-md);
  background: #2d2d2d;
  border-bottom: 1px solid #404040;
}

.code-language {
  font-size: 12px;
  color: #888;
  font-weight: 500;
}

.copy-button {
  display: flex;
  align-items: center;
  gap: var(--space-demo-xs);
  padding: 4px 8px;
  background: transparent;
  border: 1px solid #404040;
  border-radius: 4px;
  color: #888;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.copy-button:hover {
  background: #404040;
  color: #fff;
}

.copy-button svg {
  width: 12px;
  height: 12px;
}

.code-content {
  padding: var(--space-demo-md);
  margin: 0;
  color: #e6e6e6;
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 14px;
  line-height: 1.6;
  overflow-x: auto;
}

/* Footer */
.demo-footer {
  background: var(--color-demo-surface);
  border-top: 1px solid var(--color-demo-border);
  padding: var(--space-demo-xl) var(--space-demo-md);
  margin-top: var(--space-demo-3xl);
}

.footer-content {
  max-width: 1200px;
  margin: 0 auto;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-demo-md);
}

.footer-text {
  font-size: 14px;
  color: var(--color-demo-text-secondary);
  margin: 0;
}

.footer-links {
  display: flex;
  gap: var(--space-demo-md);
}

.footer-link {
  background: none;
  border: none;
  color: var(--color-demo-accent);
  font-size: 14px;
  cursor: pointer;
  text-decoration: none;
  transition: opacity 0.2s ease;
}

.footer-link:hover {
  opacity: 0.7;
}

/* Responsive Design */
@media (max-width: 768px) {
  .demo-title {
    font-size: 36px;
  }
  
  .demo-subtitle {
    font-size: 18px;
  }
  
  .section-title {
    font-size: 28px;
  }
  
  .design-principles {
    gap: var(--space-demo-md);
  }
  
  .features-grid {
    grid-template-columns: 1fr;
  }
  
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .footer-content {
    flex-direction: column;
    text-align: center;
  }
}
</style> 