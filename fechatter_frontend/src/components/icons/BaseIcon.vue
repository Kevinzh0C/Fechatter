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
    <slot />
  </svg>
</template>

<script>
export default {
  name: 'BaseIcon',
  props: {
    size: {
      type: [Number, String],
      default: 24
    },
    color: {
      type: String,
      default: 'currentColor'
    },
    startColor: {
      type: String,
      default: '#6B46C1'
    },
    endColor: {
      type: String,
      default: '#4F46E5'
    },
    preserveGradient: {
      type: Boolean,
      default: false
    },
    title: {
      type: String,
      default: ''
    },
    className: {
      type: String,
      default: ''
    },
    clickable: {
      type: Boolean,
      default: false
    }
  },
  emits: ['click'],
  computed: {
    computedSize() {
      return typeof this.size === 'number' ? `${this.size}px` : this.size;
    },
    actualStartColor() {
      return this.color !== 'currentColor' ? this.color : this.startColor;
    },
    actualEndColor() {
      return this.color !== 'currentColor' ? this.color : this.endColor;
    },
    computedStyle() {
      return {
        cursor: this.clickable ? 'pointer' : 'default',
        transition: 'all 0.2s ease'
      };
    }
  },
  methods: {
    handleClick(event) {
      if (this.clickable) {
        this.$emit('click', event);
      }
    }
  }
};
</script>

<style scoped>
.fechatter-icon {
  display: inline-block;
  vertical-align: middle;
}

.fechatter-icon:hover {
  opacity: 0.8;
}
</style>