import { ref, computed, watch } from 'vue';

/**
 * M4: 占位 Skeleton & 渐进填充
 * 滚动速度 v → 预加载区间 L = v × (网络 RTT + 2 frames)
 * 未到货的消息用固定 Skeleton，占位高度提前写进缓存
 */
export function useProgressiveLoading(options = {}) {
  const {
    networkRTT = 100, // 默认网络往返时间 (ms)
    frameTime = 16.67, // 60fps 下的帧时间
    skeletonHeight = 80, // 默认骨架屏高度
    preloadThreshold = 200, // 预加载触发阈值 (px)
    maxPreloadItems = 10 // 最大预加载项目数
  } = options;

  // 滚动状态
  const scrollVelocity = ref(0);
  const scrollDirection = ref('none'); // 'up', 'down', 'none'
  const lastScrollTop = ref(0);
  const lastScrollTime = ref(0);

  // 加载状态
  const loadingRanges = ref(new Map()); // Map<rangeKey, { start, end, status }>
  const skeletonItems = ref([]);
  const loadedItems = ref(new Map());

  // 性能指标
  const metrics = ref({
    avgLoadTime: 0,
    totalLoads: 0,
    skeletonCount: 0,
    preloadHits: 0,
    preloadMisses: 0
  });

  /**
   * 计算滚动速度和方向
   */
  const calculateScrollVelocity = (currentScrollTop) => {
    const now = performance.now();
    const timeDelta = now - lastScrollTime.value;
    
    if (timeDelta > 0) {
      const distance = currentScrollTop - lastScrollTop.value;
      const velocity = Math.abs(distance / timeDelta) * 1000; // px/s
      
      scrollVelocity.value = velocity;
      scrollDirection.value = distance > 0 ? 'down' : distance < 0 ? 'up' : 'none';
      
      lastScrollTop.value = currentScrollTop;
      lastScrollTime.value = now;
    }
  };

  /**
   * 计算预加载区域
   */
  const calculatePreloadRange = (viewportTop, viewportHeight) => {
    // L = v × (RTT + 2 frames)
    const preloadTime = networkRTT + (2 * frameTime);
    const preloadDistance = Math.min(
      scrollVelocity.value * (preloadTime / 1000),
      viewportHeight * 2 // 最多预加载2个视口高度
    );

    let preloadStart, preloadEnd;

    if (scrollDirection.value === 'down') {
      // 向下滚动，预加载下方内容
      preloadStart = viewportTop + viewportHeight - preloadThreshold;
      preloadEnd = preloadStart + preloadDistance;
    } else if (scrollDirection.value === 'up') {
      // 向上滚动，预加载上方内容
      preloadEnd = viewportTop + preloadThreshold;
      preloadStart = Math.max(0, preloadEnd - preloadDistance);
    } else {
      // 静止状态，预加载视口周围
      const center = viewportTop + viewportHeight / 2;
      preloadStart = Math.max(0, center - preloadDistance / 2);
      preloadEnd = center + preloadDistance / 2;
    }

    return { start: preloadStart, end: preloadEnd };
  };

  /**
   * 创建骨架屏项目
   */
  const createSkeletonItems = (count, startIndex = 0) => {
    const items = [];
    
    for (let i = 0; i < count; i++) {
      items.push({
        id: `skeleton-${startIndex + i}-${Date.now()}`,
        type: 'skeleton',
        height: skeletonHeight,
        index: startIndex + i,
        createdAt: Date.now()
      });
    
    metrics.value.skeletonCount += count;
    return items;
  };

  /**
   * 预加载数据
   */
  const preloadData = async (range, loadFunction) => {
    const rangeKey = `${range.start}-${range.end}`;
    
    // 检查是否已在加载
    if (loadingRanges.value.has(rangeKey)) {
      const status = loadingRanges.value.get(rangeKey).status;
      if (status === 'loading' || status === 'loaded') {
        metrics.value.preloadHits++;
        return;
      }
    
    metrics.value.preloadMisses++;
    
    // 标记为加载中
    loadingRanges.value.set(rangeKey, {
      start: range.start,
      end: range.end,
      status: 'loading',
      startTime: performance.now()
    });

    try {
      // 执行加载
      const startTime = performance.now();
      const data = await loadFunction(range);
      const loadTime = performance.now() - startTime;
      
      // 更新指标
      metrics.value.totalLoads++;
      metrics.value.avgLoadTime = 
        (metrics.value.avgLoadTime * (metrics.value.totalLoads - 1) + loadTime) / 
        metrics.value.totalLoads;
      
      // 存储加载的数据
      data.forEach(item => {
        loadedItems.value.set(item.id, item);
      });
      
      // 标记为已加载
      loadingRanges.value.set(rangeKey, {
        ...loadingRanges.value.get(rangeKey),
        status: 'loaded',
        loadTime
      });
      
      return data;
    } catch (error) {
      // 标记为失败
      loadingRanges.value.set(rangeKey, {
        ...loadingRanges.value.get(rangeKey),
        status: 'failed',
        error
      });
      
      if (import.meta.env.DEV) {
        console.error('Preload failed:', error);
      throw error;
    }
  };

  /**
   * 获取显示项目（真实数据或骨架屏）
   */
  const getDisplayItems = (requiredRange, allItems) => {
    const displayItems = [];
    const { start, end } = requiredRange;
    
    for (let i = start; i <= end; i++) {
      const item = allItems[i];
      
      if (item) {
        // 有真实数据
        displayItems.push(item);
      } else if (shouldShowSkeleton(i)) {
        // 显示骨架屏
        const skeleton = createSkeletonItems(1, i)[0];
        displayItems.push(skeleton);
      }
    
    return displayItems;
  };

  /**
   * 判断是否应该显示骨架屏
   */
  const shouldShowSkeleton = (index) => {
    // 检查该位置是否在任何加载中的范围内
    for (const [_, range] of loadingRanges.value) {
      if (range.status === 'loading' && 
          index >= range.start && 
          index <= range.end) {
        return true;
    return false;
  };

  /**
   * 清理过期的加载范围
   */
  const cleanupLoadingRanges = () => {
    const now = performance.now();
    const maxAge = 60000; // 60秒
    
    loadingRanges.value.forEach((range, key) => {
      if (range.status === 'loaded' && 
          now - range.startTime > maxAge) {
        loadingRanges.value.delete(key);
      }
    });
  };

  /**
   * 处理滚动事件
   */
  const handleScroll = (scrollTop, viewportHeight) => {
    calculateScrollVelocity(scrollTop);
    
    const preloadRange = calculatePreloadRange(scrollTop, viewportHeight);
    
    return {
      preloadRange,
      velocity: scrollVelocity.value,
      direction: scrollDirection.value
    };
  };

  /**
   * 重置状态
   */
  const reset = () => {
    scrollVelocity.value = 0;
    scrollDirection.value = 'none';
    lastScrollTop.value = 0;
    lastScrollTime.value = 0;
    loadingRanges.value.clear();
    skeletonItems.value = [];
    loadedItems.value.clear();
  };

  // 定期清理
  setInterval(cleanupLoadingRanges, 30000);

  return {
    // 状态
    scrollVelocity,
    scrollDirection,
    loadingRanges,
    skeletonItems,
    metrics,
    
    // 方法
    handleScroll,
    preloadData,
    getDisplayItems,
    createSkeletonItems,
    reset
  };
}