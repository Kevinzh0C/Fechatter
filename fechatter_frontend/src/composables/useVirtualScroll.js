import { ref, computed, watch, onMounted, onUnmounted } from 'vue';

/**
 * Virtual scrolling composable for message lists
 * Provides efficient rendering of large message lists
 */
export function useVirtualScroll({
  items,
  containerRef,
  itemHeight = 80,
  overscan = 3
}) {
  const visibleRange = ref({ start: 0, end: 0 });
  const scrollTop = ref(0);
  const containerHeight = ref(0);
  // Map<messageId|string, number>
  const measuredHeights = ref(new Map());
  const performanceMetrics = ref({
    cacheHits: 0,
    lastRenderTime: 0
  });

  // Dynamically calculate total height using message IDs for stability
  const totalHeight = computed(() => {
    let height = 0;
    for (const item of items.value) {
      const key = item?.id ?? item?.temp_id;
      height += measuredHeights.value.get(key) || itemHeight;
    }
    return height;
  });

  // Calculate which items are visible
  const virtualItems = computed(() => {
    const startTime = performance.now();

    if (!items.value.length || !containerHeight.value) {
      return [];
    }

    const visibleStart = Math.max(0, visibleRange.value.start - overscan);
    const visibleEnd = Math.min(items.value.length, visibleRange.value.end + overscan);

    const result = [];
    let offset = 0;

    // Calculate offset for items before visible range
    for (let i = 0; i < visibleStart; i++) {
      const preItem = items.value[i];
      const preKey = preItem?.id ?? preItem?.temp_id;
      offset += measuredHeights.value.get(preKey) || itemHeight;
    }

    // Create virtual items for visible range
    for (let i = visibleStart; i < visibleEnd; i++) {
      const item = items.value[i];
      const key = item?.id ?? item?.temp_id;
      const height = measuredHeights.value.get(key) || itemHeight;

      result.push({
        ...item,
        index: i,
        offset,
        height
      });

      offset += height;
    }

    performanceMetrics.value.lastRenderTime = performance.now() - startTime;
    return result;
  });

  // Update visible range based on scroll position
  const updateVisibleRange = () => {
    if (!containerHeight.value) return;

    let offset = 0;
    let start = 0;
    let end = 0;

    // Find start index
    for (let i = 0; i < items.value.length; i++) {
      const key = items.value[i]?.id ?? items.value[i]?.temp_id;
      const height = measuredHeights.value.get(key) || itemHeight;
      if (offset + height > scrollTop.value) {
        start = i;
        break;
      }
      offset += height;
    }

    // Find end index
    const viewportBottom = scrollTop.value + containerHeight.value;
    for (let i = start; i < items.value.length; i++) {
      const key = items.value[i]?.id ?? items.value[i]?.temp_id;
      const height = measuredHeights.value.get(key) || itemHeight;
      offset += height;
      if (offset >= viewportBottom) {
        end = i + 1;
        break;
      }
    }

    if (end === 0) end = items.value.length;

    visibleRange.value = { start, end };
  };

  // Handle scroll events
  const handleScroll = (event) => {
    scrollTop.value = event.target.scrollTop;
    updateVisibleRange();
  };

  // Set measured height for an item
  const setMeasuredHeight = (itemId, height) => {
    if (!itemId) return;
    measuredHeights.value.set(itemId, height);
    updateVisibleRange();
  };

  // Scroll to specific item
  const scrollToItem = (index, position = 'start') => {
    if (!containerRef.value || index < 0 || index >= items.value.length) return;

    let offset = 0;
    for (let i = 0; i < index; i++) {
      const keyIter = items.value[i]?.id ?? items.value[i]?.temp_id;
      offset += measuredHeights.value.get(keyIter) || itemHeight;
    }

    const targetKey = items.value[index]?.id ?? items.value[index]?.temp_id;
    const currentItemHeight = measuredHeights.value.get(targetKey) || itemHeight;

    if (position === 'end') {
      offset += currentItemHeight - containerHeight.value;
    } else if (position === 'center') {
      offset += currentItemHeight / 2 - containerHeight.value / 2;
    }

    containerRef.value.scrollTop = Math.max(0, offset);
  };

  // Update container height
  const updateContainerHeight = () => {
    if (containerRef.value) {
      containerHeight.value = containerRef.value.clientHeight;
      updateVisibleRange();
    }
  };

  // Watch for items changes
  watch(items, () => {
    // Clear measured heights when items change significantly
    if (measuredHeights.value.size > items.value.length * 1.5) {
      measuredHeights.value.clear();
    }
    updateVisibleRange();
  }, { deep: true });

  // Setup event listeners
  onMounted(() => {
    if (containerRef.value) {
      containerRef.value.addEventListener('scroll', handleScroll, { passive: true });

      // Initial setup
      updateContainerHeight();

      // Watch for container resize
      const resizeObserver = new ResizeObserver(updateContainerHeight);
      resizeObserver.observe(containerRef.value);

      // Cleanup
      onUnmounted(() => {
        if (containerRef.value) {
          containerRef.value.removeEventListener('scroll', handleScroll);
        }
        resizeObserver.disconnect();
      });
    }
  });

  return {
    virtualItems,
    totalHeight,
    visibleRange,
    scrollToItem,
    setMeasuredHeight,
    performanceMetrics
  };
}