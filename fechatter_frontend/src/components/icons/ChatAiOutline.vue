<template>
  <svg :width="computedSize" :height="computedSize" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"
    :style="computedStyle" :class="['fechatter-icon', className]" :aria-hidden="!title" role="img" @click="handleClick">
    <title v-if="title">{{ title }}</title>
    <defs>
      <linearGradient id="chatAiOutlineGradient" x1="0%" y1="0%" x2="100%" y2="0%">
        <stop offset="0%" :stop-color="actualStartColor" />
        <stop offset="100%" :stop-color="actualEndColor" />
      </linearGradient>
    </defs>
    <path
      d="M12 2C6.48 2 2 6.48 2 12c0 1.33.26 2.61.76 3.79.13.31.12.68-.04.98l-1.08 2.16c-.18.35-.13.78.12 1.09.25.31.65.46 1.04.39l3.02-.53c.2-.03.41-.01.6.07C7.92 20.6 9.91 21 12 21c5.52 0 10-4.48 10-10S17.52 2 12 2zm0 18c-1.74 0-3.41-.34-4.94-.96l-.71-.25-.73.13-1.76.31.35-.71.16-.33.08-.35c.17-.74.13-1.5-.11-2.21-.38-.92-.59-1.92-.59-2.96 0-4.42 3.58-8 8-8s8 3.58 8 8-3.58 8-8 8z"
      fill="none" :stroke="preserveGradient ? 'url(#chatAiOutlineGradient)' : actualStartColor" stroke-width="1.5" />
    <g :fill="preserveGradient ? 'url(#chatAiOutlineGradient)' : actualStartColor">
      <circle cx="12" cy="12" r="1.5" />
      <circle cx="9" cy="9" r="1" />
      <circle cx="15" cy="9" r="1" />
      <circle cx="9" cy="15" r="1" />
      <circle cx="15" cy="15" r="1" />
      <circle cx="7.5" cy="12" r="1" />
      <circle cx="16.5" cy="12" r="1" />
      <path d="M9 9l3 3M15 9l-3 3M9 15l3-3M15 15l-3-3M7.5 12H9M15 12h1.5"
        :stroke="preserveGradient ? 'url(#chatAiOutlineGradient)' : actualStartColor" stroke-width="0.75" />
    </g>
  </svg>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import BaseIcon from './BaseIcon.vue';

export default defineComponent({
  name: 'ChatAiOutline',
  extends: BaseIcon
});
</script>

<style scoped>
.fechatter-icon {
  display: inline-block;
  vertical-align: middle;
}
</style>
