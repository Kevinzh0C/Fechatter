import { ref, computed, watch, onMounted, onUnmounted } from 'vue';

/**
 * M3: 窗口虚拟化 + 高度缓存
 * 仅渲染可视 ± N 条，消息高度进 Map<id, h>；滚动计算依靠累计高度
 */
export function useVirtualScroll(options = {}) {
  const {
    items = ref([]),
    containerRef = ref(null),
    itemHeight = 80, // 默认项目高度
    overscan = 5, // 视口外额外渲染的项目数
    minItemHeight = 40,
    maxItemHeight = 400
  } = options;

  // 高度缓存 Map
  const heightCache = new Map();
  const measuredHeights = ref(new Map());
  
  // 虚拟化状态
  const scrollTop = ref(0);
  const containerHeight = ref(0);
  const totalHeight = ref(0);
  const visibleRange = ref({ start: 0, end: 0 });
  
  // 性能监控
  const performanceMetrics = ref({
    renderCount: 0,
    cacheHits: 0,
    cacheMisses: 0,
    avgMeasureTime: 0,
    lastRenderTime: 0
  });

  // 获取或估算项目高度
  const getItemHeight = (itemId) => {
    if (heightCache.has(itemId)) {
      performanceMetrics.value.cacheHits++;
      return heightCache.get(itemId);
    }
    
    performanceMetrics.value.cacheMisses++;
    return itemHeight; // 返回默认高度
  };

  // 设置实测高度
  const setMeasuredHeight = (itemId, height) => {
    const clampedHeight = Math.max(minItemHeight, Math.min(maxItemHeight, height));
    
    if (heightCache.get(itemId) !== clampedHeight) {
      heightCache.set(itemId, clampedHeight);
      measuredHeights.value.set(itemId, clampedHeight);
      recalculateTotalHeight();
    }
  };

  // 计算总高度
  const recalculateTotalHeight = () => {
    let total = 0;
    items.value.forEach(item => {
      const id = item.id || item.temp_id;
      total += getItemHeight(id);
    });
    totalHeight.value = total;
  };

  // 二分查找可视范围
  const findVisibleRange = () => {
    const container = containerRef.value;
    if (!container || items.value.length === 0) {
      return { start: 0, end: 0 };
    }

    const scrollOffset = scrollTop.value;
    const viewportHeight = containerHeight.value;
    
    let accumulatedHeight = 0;
    let startIndex = 0;
    let endIndex = items.value.length - 1;

    // 找到起始索引
    for (let i = 0; i < items.value.length; i++) {
      const item = items.value[i];
      const height = getItemHeight(item.id || item.temp_id);
      
      if (accumulatedHeight + height > scrollOffset) {
        startIndex = Math.max(0, i - overscan);
        break;
      }
      accumulatedHeight += height;
    }

    // 找到结束索引
    accumulatedHeight = 0;
    for (let i = 0; i < items.value.length; i++) {
      const item = items.value[i];
      const height = getItemHeight(item.id || item.temp_id);
      accumulatedHeight += height;
      
      if (accumulatedHeight > scrollOffset + viewportHeight) {
        endIndex = Math.min(items.value.length - 1, i + overscan);
        break;
      }
    }

    return { start: startIndex, end: endIndex };
  };

  // 计算项目偏移量
  const getItemOffset = (index) => {
    let offset = 0;
    for (let i = 0; i < index; i++) {
      const item = items.value[i];
      offset += getItemHeight(item.id || item.temp_id);
    }
    return offset;
  };

  // 虚拟项目列表
  const virtualItems = computed(() => {
    const range = visibleRange.value;
    const result = [];

    for (let i = range.start; i <= range.end; i++) {
      const item = items.value[i];
      if (!item) continue;

      const id = item.id || item.temp_id;
      result.push({
        ...item,
        index: i,
        offset: getItemOffset(i),
        height: getItemHeight(id),
        isVirtual: true
      });
    }

    return result;
  });

  // 处理滚动
  const handleScroll = () => {
    const container = containerRef.value;
    if (!container) return;

    scrollTop.value = container.scrollTop;
    visibleRange.value = findVisibleRange();
  };

  // 处理容器大小变化
  const handleResize = () => {
    const container = containerRef.value;
    if (!container) return;

    containerHeight.value = container.clientHeight;
    visibleRange.value = findVisibleRange();
  };

  // 滚动到指定项目
  const scrollToItem = (index, alignment = 'start') => {
    const container = containerRef.value;
    if (!container || index < 0 || index >= items.value.length) return;

    const offset = getItemOffset(index);
    const item = items.value[index];
    const height = getItemHeight(item.id || item.temp_id);

    let scrollPosition = offset;

    if (alignment === 'center') {
      scrollPosition = offset - (containerHeight.value - height) / 2;
    } else if (alignment === 'end') {
      scrollPosition = offset - containerHeight.value + height;
    }

    container.scrollTop = Math.max(0, scrollPosition);
  };

  // 测量所有可见项目
  const measureVisibleItems = () => {
    const container = containerRef.value;
    if (!container) return;

    const startTime = performance.now();
    const elements = container.querySelectorAll('[data-virtual-item]');
    
    elements.forEach(el => {
      const itemId = el.getAttribute('data-item-id');
      const rect = el.getBoundingClientRect();
      
      if (itemId && rect.height > 0) {
        setMeasuredHeight(itemId, rect.height);
      }
    });

    const elapsed = performance.now() - startTime;
    performanceMetrics.value.avgMeasureTime = 
      (performanceMetrics.value.avgMeasureTime + elapsed) / 2;
  };

  // 清理不再需要的高度缓存
  const cleanupCache = () => {
    const currentIds = new Set(items.value.map(item => item.id || item.temp_id));
    
    heightCache.forEach((_, id) => {
      if (!currentIds.has(id)) {
        heightCache.delete(id);
        measuredHeights.value.delete(id);
      }
    });
  };

  // 监听项目变化
  watch(items, () => {
    recalculateTotalHeight();
    visibleRange.value = findVisibleRange();
    cleanupCache();
  }, { deep: true });

  // 监听滚动位置变化
  watch(scrollTop, () => {
    visibleRange.value = findVisibleRange();
  });

  // 设置滚动和大小监听
  let scrollRAF = null;
  let resizeObserver = null;

  onMounted(() => {
    const container = containerRef.value;
    if (!container) return;

    // 滚动监听（使用 RAF 节流）
    const onScroll = () => {
      if (scrollRAF) cancelAnimationFrame(scrollRAF);
      scrollRAF = requestAnimationFrame(handleScroll);
    };
    container.addEventListener('scroll', onScroll, { passive: true });

    // 大小监听
    resizeObserver = new ResizeObserver(handleResize);
    resizeObserver.observe(container);

    // 初始化
    handleResize();
    handleScroll();

    // 定期测量可见项目
    const measureInterval = setInterval(measureVisibleItems, 1000);

    // 清理函数
    onUnmounted(() => {
      container.removeEventListener('scroll', onScroll);
      if (resizeObserver) resizeObserver.disconnect();
      if (scrollRAF) cancelAnimationFrame(scrollRAF);
      clearInterval(measureInterval);
    });
  });

  return {
    virtualItems,
    totalHeight,
    visibleRange,
    scrollToItem,
    setMeasuredHeight,
    measureVisibleItems,
    performanceMetrics,
    getItemOffset
  };
}