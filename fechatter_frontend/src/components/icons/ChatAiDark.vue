<template>
  <svg
    :width="computedSize"
    :height="computedSize"
    viewBox="0 0 24 24"
    xmlns="http://www.w3.org/2000/svg"
    :style="computedStyle"
    :class="['fechatter-icon', className]"
    aria-hidden="!title"
    role="img"
    @click="handleClick"
  >
    <title v-if="title">{{ title }}</title>
    <defs>
      <linearGradient id="chatAiDarkGradient" x1="0%" y1="0%" x2="100%" y2="0%">
        <stop offset="0%" :stop-color="actualStartColor" />
        <stop offset="100%" :stop-color="actualEndColor" />
      </linearGradient>
    </defs>
    <rect x="3" y="3" width="18" height="18" rx="6" fill="#121212" />
    <path d="M19 9c0-3.87-3.13-7-7-7S5 5.13 5 9c0 1.74.64 3.33 1.69 4.54L5 19l5.46-1.69c.66.34 1.4.54 2.17.62.77.08 1.55.04 2.31-.13.76-.16 1.48-.44 2.13-.82.65-.38 1.23-.86 1.69-1.42.47-.56.83-1.19 1.08-1.86.25-.67.37-1.37.37-2.07.01-.56-.09-1.13-.21-1.63zm-7 5c-.83 0-1.58-.34-2.12-.88S9 11.83 9 11c0-.83.34-1.58.88-2.12S11.17 8 12 8c.83 0 1.58.34 2.12.88.54.54.88 1.29.88 2.12 0 .83-.34 1.58-.88 2.12-.54.54-1.29.88-2.12.88z" fill="none" :stroke="preserveGradient ? 'url(#chatAiDarkGradient)' : actualStartColor" stroke-width="1.5" />
    <g :fill="preserveGradient ? 'url(#chatAiDarkGradient)' : actualStartColor">
      <circle cx="12" cy="11" r="1.5" />
      <circle cx="9" cy="8" r="1" />
      <circle cx="15" cy="8" r="1" />
      <circle cx="9" cy="14" r="1" />
      <circle cx="15" cy="14" r="1" />
      <circle cx="7.5" cy="11" r="1" />
      <circle cx="16.5" cy="11" r="1" />
      <path d="M9 8l3 3M15 8l-3 3M9 14l3-3M15 14l-3-3M7.5 11H9M15 11h1.5" :stroke="preserveGradient ? 'url(#chatAiDarkGradient)' : actualStartColor" stroke-width="0.75" />
    </g>
  </svg>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import BaseIcon from './BaseIcon';

export default defineComponent({
  name: 'ChatAiDark',
  extends: BaseIcon
});
</script>

<style scoped>
.fechatter-icon {
  display: inline-block;
  vertical-align: middle;
}
</style>
