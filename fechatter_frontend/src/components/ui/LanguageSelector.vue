<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-75 ease-out"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition duration-50 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="visible"
        ref="selectorRef"
        class="language-selector"
        :style="{
          top: `${position.top}px`,
          left: `${position.left}px`
        }"
        @keydown="handleKeyDown"
      >
        <!-- Search Input -->
        <div class="p-2 border-b border-gray-100">
          <input
            ref="searchInput"
            v-model="searchQuery"
            type="text"
            placeholder="Search languages..."
            class="w-full px-3 py-1.5 text-sm bg-gray-50 border border-gray-200 rounded-md focus:outline-none focus:ring-2 focus:ring-violet-400/50 focus:border-violet-400"
            @keydown.stop
            @keydown.down.prevent="navigateDown"
            @keydown.up.prevent="navigateUp"
            @keydown.enter.prevent="selectCurrent"
            @keydown.esc="$emit('close')"
          />
        </div>

        <!-- Language List -->
        <div class="max-h-64 overflow-y-auto">
          <!-- Suggested Languages -->
          <div v-if="!searchQuery && suggestedLanguages.length > 0" class="p-2">
            <div class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-1 px-2">
              Suggested
            </div>
            <button
              v-for="(lang, index) in suggestedLanguages"
              :key="`suggested-${lang.code}`"
              :ref="el => setItemRef(el, index)"
              class="language-item"
              :class="{ 'active': selectedIndex === index }"
              @click="selectLanguage(lang)"
              @mouseenter="selectedIndex = index"
            >
              <span class="font-medium">{{ lang.name }}</span>
              <span class="text-xs text-gray-400">{{ lang.code }}</span>
            </button>
          </div>

          <!-- Search Results / All Languages -->
          <div class="p-2" :class="{ 'border-t border-gray-100': !searchQuery && suggestedLanguages.length > 0 }">
            <div v-if="searchQuery" class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-1 px-2">
              Results
            </div>
            <div v-else-if="suggestedLanguages.length > 0" class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-1 px-2">
              All Languages
            </div>

            <button
              v-for="(lang, index) in filteredLanguages"
              :key="lang.code"
              :ref="el => setItemRef(el, suggestedOffset + index)"
              class="language-item"
              :class="{ 'active': selectedIndex === suggestedOffset + index }"
              @click="selectLanguage(lang)"
              @mouseenter="selectedIndex = suggestedOffset + index"
            >
              <span class="font-medium">{{ lang.name }}</span>
              <span class="text-xs text-gray-400">{{ lang.code }}</span>
              <span v-if="lang.aliases.length > 0" class="text-xs text-gray-300 ml-1">
                ({{ lang.aliases.join(', ') }})
              </span>
            </button>

            <!-- No Results -->
            <div v-if="searchQuery && filteredLanguages.length === 0" class="px-3 py-2 text-sm text-gray-500 text-center">
              No languages found
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="p-2 border-t border-gray-100 bg-gray-50/50">
          <div class="flex items-center justify-between text-xs text-gray-500">
            <div class="flex items-center space-x-3">
              <span class="flex items-center space-x-1">
                <kbd class="kbd">↑↓</kbd>
                <span>Navigate</span>
              </span>
              <span class="flex items-center space-x-1">
                <kbd class="kbd">Enter</kbd>
                <span>Select</span>
              </span>
              <span class="flex items-center space-x-1">
                <kbd class="kbd">Esc</kbd>
                <span>Cancel</span>
              </span>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';

const props = defineProps({
  visible: {
    type: Boolean,
    required: true
  },
  position: {
    type: Object,
    required: true,
    default: () => ({ top: 0, left: 0 })
  },
  languages: {
    type: Array,
    required: true
  },
  suggestedLanguages: {
    type: Array,
    default: () => []
  }
});

const emit = defineEmits(['select', 'close']);

// State
const searchQuery = ref('');
const selectedIndex = ref(0);
const itemRefs = ref([]);
const searchInput = ref(null);
const selectorRef = ref(null);

// Computed
const filteredLanguages = computed(() => {
  if (!searchQuery.value) {
    return props.languages;
  }
  
  const query = searchQuery.value.toLowerCase();
  return props.languages.filter(lang =>
    lang.name.toLowerCase().includes(query) ||
    lang.code.toLowerCase().includes(query) ||
    lang.aliases.some(alias => alias.toLowerCase().includes(query))
  );
});

const suggestedOffset = computed(() => 
  !searchQuery.value && props.suggestedLanguages.length > 0 
    ? props.suggestedLanguages.length 
    : 0
);

const totalItems = computed(() => 
  suggestedOffset.value + filteredLanguages.value.length
);

// Methods
function setItemRef(el, index) {
  if (el) {
    itemRefs.value[index] = el;
  }
}

function navigateUp() {
  if (selectedIndex.value > 0) {
    selectedIndex.value--;
    scrollToSelected();
  }
}

function navigateDown() {
  if (selectedIndex.value < totalItems.value - 1) {
    selectedIndex.value++;
    scrollToSelected();
  }
}

function scrollToSelected() {
  nextTick(() => {
    const selected = itemRefs.value[selectedIndex.value];
    if (selected) {
      selected.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  });
}

function selectCurrent() {
  if (selectedIndex.value < suggestedOffset.value) {
    selectLanguage(props.suggestedLanguages[selectedIndex.value]);
  } else {
    const index = selectedIndex.value - suggestedOffset.value;
    if (index < filteredLanguages.value.length) {
      selectLanguage(filteredLanguages.value[index]);
    }
  }
}

function selectLanguage(language) {
  emit('select', language);
  resetState();
}

function resetState() {
  searchQuery.value = '';
  selectedIndex.value = 0;
  itemRefs.value = [];
}

function handleKeyDown(event) {
  // Prevent default browser behavior for navigation keys
  if (['ArrowUp', 'ArrowDown', 'Enter', 'Escape'].includes(event.key)) {
    event.preventDefault();
  }
}

// Handle click outside
function handleClickOutside(event) {
  if (selectorRef.value && !selectorRef.value.contains(event.target)) {
    emit('close');
  }
}

// Lifecycle
watch(() => props.visible, (newVal) => {
  if (newVal) {
    resetState();
    nextTick(() => {
      searchInput.value?.focus();
      document.addEventListener('click', handleClickOutside);
    });
  } else {
    document.removeEventListener('click', handleClickOutside);
  }
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<style scoped>
.language-selector {
  @apply fixed z-50 w-80 bg-white rounded-lg shadow-lg border border-gray-200 overflow-hidden;
  /* Custom positioning will be applied inline */
}

.language-item {
  @apply w-full px-3 py-2 text-left flex items-center justify-between hover:bg-gray-50 transition-colors text-sm;
}

.language-item.active {
  @apply bg-violet-50 text-violet-700;
}

.language-item.active .text-gray-400 {
  @apply text-violet-500;
}

.language-item.active .text-gray-300 {
  @apply text-violet-400;
}

.kbd {
  @apply inline-flex items-center justify-center px-1.5 py-0.5 text-xs font-medium bg-white border border-gray-300 rounded;
}

/* Smooth scrollbar */
.language-selector ::-webkit-scrollbar {
  width: 6px;
}

.language-selector ::-webkit-scrollbar-track {
  background: transparent;
}

.language-selector ::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.3);
  border-radius: 3px;
}

.language-selector ::-webkit-scrollbar-thumb:hover {
  background: rgba(156, 163, 175, 0.5);
}
</style>