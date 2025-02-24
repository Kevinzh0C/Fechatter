<template>
  <div class="code-block-container" :class="`language-${language}`">
    <div v-if="highlightedCode" v-html="highlightedCode" class="shiki"></div>
    <div v-else class="preformatted-code">
      <pre><code>{{ code }}</code></pre>
    </div>
  </div>
</template>

<script setup>
import { ref, watchEffect } from 'vue';
import { getShiki } from '@/services/shiki';

const props = defineProps({
  code: {
    type: String,
    required: true,
  },
  language: {
    type: String,
    default: 'text',
  },
});

const highlightedCode = ref('');

watchEffect(async () => {
  try {
    const shiki = await getShiki();
    highlightedCode.value = shiki.codeToHtml(props.code, {
      lang: props.language,
      theme: 'github-light',
    });
  } catch (error) {
    console.error('Failed to highlight code:', error);
    // Fallback to plain text
    highlightedCode.value = `<pre><code>${props.code}</code></pre>`;
  }
});
</script>

<style>
/* Add styles for your code blocks here */
.code-block-container .shiki {
  background-color: #f6f8fa;
  border-radius: 6px;
  padding: 16px;
  overflow-x: auto;
  font-family: 'Courier New', Courier, monospace;
}
</style>