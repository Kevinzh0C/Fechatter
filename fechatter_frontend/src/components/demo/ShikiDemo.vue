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
import { ref, onMounted, computed, nextTick } from 'vue';
// Remove static import to avoid conflicts with dynamic imports
// import { highlightSingleCodeBlock, generateHighlightStyles, switchTheme } from '@/utils/codeHighlight.js';

// Demo state
const selectedLanguage = ref('javascript');
const selectedTheme = ref('github-dark');
const codeInput = ref(`function fibonacci(n) {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}`);

const highlightedOutput = ref('');
const isLoading = ref(false);
const error = ref(null);

// Available options
const languages = ref([
  'javascript', 'typescript', 'python', 'rust', 'go', 'java', 'cpp', 'css', 'html', 'json'
]);

const themes = ref([
  'github-dark', 'github-light', 'dracula', 'monokai', 'nord', 'one-dark-pro'
]);

// Cached highlight functions
let highlightSingleCodeBlock = null;
let generateHighlightStyles = null;
let switchTheme = null;

/**
 * Initialize highlight functions
 */
const initializeHighlighter = async () => {
  try {
    const codeHighlightModule = await import('@/utils/codeHighlight.js');
    
    highlightSingleCodeBlock = codeHighlightModule.highlightSingleCodeBlock;
    generateHighlightStyles = codeHighlightModule.generateHighlightStyles;
    switchTheme = codeHighlightModule.switchTheme;
    
    return true;
  } catch (err) {
    console.error('Failed to load code highlight module:', err);
    error.value = err;
    return false;
  }
};

// Methods
async function highlightCode() {
  if (!codeInput.value.trim()) {
    highlightedOutput.value = '';
    return;
  }

  isLoading.value = true;
  error.value = null;

  try {
    // Ensure highlighter is initialized
    if (!highlightSingleCodeBlock) {
      const initialized = await initializeHighlighter();
      if (!initialized) {
        throw new Error('Failed to initialize highlighter');
      }
    }

    const result = await highlightSingleCodeBlock(
      codeInput.value,
      selectedLanguage.value,
      {
        theme: selectedTheme.value,
        lineNumbers: true,
      }
    );

    highlightedOutput.value = result;
  } catch (err) {
    console.error('Highlighting failed:', err);
    error.value = err;
    highlightedOutput.value = `<pre><code>${codeInput.value}</code></pre>`;
  } finally {
    isLoading.value = false;
  }
}

async function changeTheme() {
  try {
    // Ensure theme switcher is initialized
    if (!switchTheme) {
      const initialized = await initializeHighlighter();
      if (!initialized) {
        throw new Error('Failed to initialize theme switcher');
      }
    }

    await switchTheme(selectedTheme.value);
    
    // Re-highlight with new theme
    await highlightCode();
  } catch (err) {
    console.error('Theme change failed:', err);
    error.value = err;
  }
}

async function processMarkdownDemo() {
  const markdownContent = `
# Code Demo

Here's some JavaScript:

\`\`\`javascript
${codeInput.value}
\`\`\`

And some Python:

\`\`\`python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)
\`\`\`
  `;

  try {
    // Load markdown processor dynamically
    const { processMarkdownWithHighlight } = await import('@/utils/codeHighlight.js');
    
    const result = await processMarkdownWithHighlight(markdownContent, {
      theme: selectedTheme.value,
    });

    highlightedOutput.value = result;
  } catch (err) {
    console.error('Markdown processing failed:', err);
    error.value = err;
  }
}

// Computed
const isValidCode = computed(() => codeInput.value.trim().length > 0);

// Lifecycle
onMounted(async () => {
  await initializeHighlighter();
  await highlightCode();
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