<template>
  <div class="shiki-demo">
    <h2 class="text-2xl font-bold mb-4">Shiki Syntax Highlighting Demo</h2>
    
    <!-- Theme Switcher -->
    <div class="mb-4">
      <label class="flex items-center space-x-2">
        <span>Theme:</span>
        <select v-model="currentTheme" @change="onThemeChange" class="border rounded px-2 py-1">
          <option value="dark">Dark (One Dark Pro)</option>
          <option value="light">Light (Vitesse Light)</option>
        </select>
      </label>
    </div>

    <!-- Code Examples -->
    <div class="space-y-6">
      <!-- JavaScript Example -->
      <div class="example">
        <h3 class="text-lg font-semibold mb-2">JavaScript with Line Highlighting</h3>
        <div v-html="javascriptExample" class="markdown-content"></div>
      </div>

      <!-- Vue Example -->
      <div class="example">
        <h3 class="text-lg font-semibold mb-2">Vue Component</h3>
        <div v-html="vueExample" class="markdown-content"></div>
      </div>

      <!-- Rust Example -->
      <div class="example">
        <h3 class="text-lg font-semibold mb-2">Rust with Title</h3>
        <div v-html="rustExample" class="markdown-content"></div>
      </div>

      <!-- Multiple Languages -->
      <div class="example">
        <h3 class="text-lg font-semibold mb-2">Multiple Languages</h3>
        <div v-html="multiLanguageExample" class="markdown-content"></div>
      </div>

      <!-- Live Editor -->
      <div class="example">
        <h3 class="text-lg font-semibold mb-2">Live Editor</h3>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block mb-2">
              <span class="text-sm font-medium">Language:</span>
              <input 
                v-model="liveLanguage" 
                type="text" 
                class="w-full border rounded px-2 py-1 mt-1"
                placeholder="javascript"
              >
            </label>
            <label class="block mb-2">
              <span class="text-sm font-medium">Metadata:</span>
              <input 
                v-model="liveMeta" 
                type="text" 
                class="w-full border rounded px-2 py-1 mt-1"
                placeholder='{1,3-5} title="example.js"'
              >
            </label>
            <textarea
              v-model="liveCode"
              class="w-full h-64 border rounded p-2 font-mono text-sm"
              placeholder="Enter your code here..."
            ></textarea>
          </div>
          <div>
            <div v-if="liveHighlighted" v-html="liveHighlighted" class="markdown-content"></div>
            <div v-else class="text-gray-500 italic">Enter code to see highlighted output</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Performance Stats -->
    <div v-if="stats.length > 0" class="mt-6 p-4 bg-gray-100 dark:bg-gray-800 rounded">
      <h3 class="text-lg font-semibold mb-2">Performance Stats</h3>
      <ul class="space-y-1 text-sm">
        <li v-for="stat in stats" :key="stat.id">
          {{ stat.language }}: {{ stat.time.toFixed(2) }}ms
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, nextTick } from 'vue';
import { highlightSingleCodeBlock, generateHighlightStyles, switchTheme } from '@/utils/codeHighlight.js';

// State
const currentTheme = ref('dark');
const javascriptExample = ref('');
const vueExample = ref('');
const rustExample = ref('');
const multiLanguageExample = ref('');
const liveCode = ref('');
const liveLanguage = ref('javascript');
const liveMeta = ref('{2,4-6}');
const liveHighlighted = ref('');
const stats = ref([]);

// Code samples
const samples = {
  javascript: {
    code: `// Fibonacci sequence generator
function* fibonacci(limit = 10) {
  let [prev, curr] = [0, 1];
  
  for (let i = 0; i < limit; i++) {
    yield curr;
    [prev, curr] = [curr, prev + curr];
  }
}

// Usage example
const fib = fibonacci(10);
for (const num of fib) {
  console.log(num);
}`,
    meta: '{2,4-6}'
  },
  
  vue: {
    code: `<template>
  <div class="hello-world">
    <h1>{{ greeting }}</h1>
    <button @click="updateGreeting">
      Change Greeting
    </button>
  </div>
</template>

<script setup>
import { ref } from 'vue';

const greeting = ref('Hello, World!');

function updateGreeting() {
  greeting.value = 'Hello, Vue 3!';
}
</script>`,
    meta: 'title="HelloWorld.vue"'
  },
  
  rust: {
    code: `use std::collections::HashMap;

// Define a simple cache structure
struct Cache<T> {
    store: HashMap<String, T>,
    max_size: usize,
}

impl<T> Cache<T> {
    fn new(max_size: usize) -> Self {
        Self {
            store: HashMap::new(),
            max_size,
        }
    }
    
    fn get(&self, key: &str) -> Option<&T> {
        self.store.get(key)
    }
    
    fn set(&mut self, key: String, value: T) {
        if self.store.len() >= self.max_size {
            // Simple eviction: remove first item
            if let Some(first_key) = self.store.keys().next().cloned() {
                self.store.remove(&first_key);
            }
        }
        self.store.insert(key, value);
    }
}`,
    meta: 'title="cache.rs" {9-14,20-27}'
  }
};

// Multi-language example markdown
const multiLanguageMarkdown = `
### Configuration Example

First, set up your \`package.json\`:

\`\`\`json title="package.json"
{
  "name": "my-app",
  "version": "1.0.0",
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  }
}
\`\`\`

Then create a configuration file:

\`\`\`yaml title="config.yml" {3-5}
server:
  host: localhost
  port: 3000
  ssl: true
  cert: ./certs/server.crt

database:
  type: postgres
  host: localhost
\`\`\`

Finally, implement the server:

\`\`\`typescript title="server.ts"
import express from 'express';
import { loadConfig } from './config';

const app = express();
const config = loadConfig();

app.listen(config.server.port, () => {
  console.log(\`Server running on port \${config.server.port}\`);
});
\`\`\`
`;

// Initialize examples
async function initializeExamples() {
  const startTime = performance.now();
  
  // Highlight JavaScript
  const jsStart = performance.now();
  javascriptExample.value = await highlightSingleCodeBlock(
    samples.javascript.code,
    'javascript',
    samples.javascript.meta,
    { theme: currentTheme.value }
  );
  stats.value.push({ id: 'js', language: 'JavaScript', time: performance.now() - jsStart });
  
  // Highlight Vue
  const vueStart = performance.now();
  vueExample.value = await highlightSingleCodeBlock(
    samples.vue.code,
    'vue',
    samples.vue.meta,
    { theme: currentTheme.value }
  );
  stats.value.push({ id: 'vue', language: 'Vue', time: performance.now() - vueStart });
  
  // Highlight Rust
  const rustStart = performance.now();
  rustExample.value = await highlightSingleCodeBlock(
    samples.rust.code,
    'rust',
    samples.rust.meta,
    { theme: currentTheme.value }
  );
  stats.value.push({ id: 'rust', language: 'Rust', time: performance.now() - rustStart });
  
  // Process markdown with multiple languages
  const mdStart = performance.now();
  const { processMarkdownWithHighlight } = await import('@/utils/codeHighlight.js');
  multiLanguageExample.value = await processMarkdownWithHighlight(multiLanguageMarkdown, {
    theme: currentTheme.value
  });
  stats.value.push({ id: 'md', language: 'Markdown (multi)', time: performance.now() - mdStart });
  
  console.log('Total initialization time:', performance.now() - startTime, 'ms');
}

// Theme change handler
async function onThemeChange() {
  // Update styles
  const newStyles = await switchTheme(currentTheme.value);
  updateStyles(newStyles);
  
  // Re-highlight all examples
  stats.value = [];
  await initializeExamples();
  if (liveCode.value) {
    await highlightLiveCode();
  }
}

// Update styles dynamically
function updateStyles(styles) {
  let styleEl = document.getElementById('shiki-dynamic-styles');
  if (!styleEl) {
    styleEl = document.createElement('style');
    styleEl.id = 'shiki-dynamic-styles';
    document.head.appendChild(styleEl);
  }
  styleEl.textContent = styles;
}

// Live code highlighting
async function highlightLiveCode() {
  if (!liveCode.value) {
    liveHighlighted.value = '';
    return;
  }
  
  try {
    const start = performance.now();
    liveHighlighted.value = await highlightSingleCodeBlock(
      liveCode.value,
      liveLanguage.value || 'plaintext',
      liveMeta.value,
      { theme: currentTheme.value }
    );
    console.log('Live highlight time:', performance.now() - start, 'ms');
  } catch (error) {
    console.error('Live highlighting error:', error);
    liveHighlighted.value = `<pre class="error">Error: ${error.message}</pre>`;
  }
}

// Watch for live code changes
let debounceTimer;
watch([liveCode, liveLanguage, liveMeta], () => {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(highlightLiveCode, 300);
});

// Initialize on mount
onMounted(async () => {
  // Add initial styles
  const styles = generateHighlightStyles(currentTheme.value);
  updateStyles(styles);
  
  // Initialize examples
  await initializeExamples();
  
  // Set initial live code
  liveCode.value = samples.javascript.code;
  await highlightLiveCode();
});
</script>

<style scoped>
.shiki-demo {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}

.example {
  background: var(--bg-secondary);
  padding: 1.5rem;
  border-radius: 0.5rem;
  border: 1px solid var(--border-color);
}

.markdown-content {
  font-size: 0.875rem;
}

/* Dark mode support */
:root {
  --bg-secondary: #f9fafb;
  --border-color: #e5e7eb;
}

.dark {
  --bg-secondary: #1f2937;
  --border-color: #374151;
}

.error {
  color: #ef4444;
  background: #fee;
  padding: 1rem;
  border-radius: 0.25rem;
}
</style>