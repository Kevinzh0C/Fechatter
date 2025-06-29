import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import api from '@/services/api';

/**
 * M1: 统一搜索队列 + debounce
 * 防止连续击键打爆后端；300 ms collect → /search?query=…&chat_id=…&cursor=…
 */
export function useChatSearch(chatId) {
  // 搜索状态
  const searchQuery = ref('');
  const searchResults = ref([]);
  const isSearching = ref(false);
  const searchError = ref(null);
  const hasMore = ref(false);
  const currentCursor = ref(null);

  // 搜索历史（短期记忆优化）
  const MAX_RECENT_SEARCHES = 7; // 人最多记 7±2 信息
  const recentSearches = ref(JSON.parse(localStorage.getItem('recentSearches') || '[]'));
  const searchPresets = ref([
    { label: 'Images', filter: 'type:image' },
    { label: 'Links', filter: 'type:link' },
    { label: 'Files', filter: 'type:file' },
    { label: 'From me', filter: 'from:me' },
    { label: 'Mentions', filter: 'mentions:me' }
  ]);

  // 性能监控
  const performanceMetrics = ref({
    searchCount: 0,
    avgResponseTime: 0,
    lastSearchTime: 0,
    debounceCount: 0
  });

  // 搜索配置
  const DEBOUNCE_DELAY = 300; // ms
  const MIN_QUERY_LENGTH = 2;
  const PAGE_SIZE = 50; // API必须200ms内返回50条
  const SEARCH_TIMEOUT = 5000; // 5s超时

  // Debounce定时器
  let debounceTimer = null;
  let searchAbortController = null;

  // Simple SearchService implementation
  const SearchService = {
    async search(params) {
      try {
        const response = await api.get('/search/messages', { params });
        return response.data || { results: [], total: 0 };
      } catch (error) {
        console.error('Search error:', error);
        return { results: [], total: 0 };
      }
    }
  };

  /**
   * 执行搜索
   */
  const performSearch = async (query, cursor = null) => {
    // 取消之前的搜索
    if (searchAbortController) {
      searchAbortController.abort();
    }

    // 验证查询
    if (!query || query.trim().length < MIN_QUERY_LENGTH) {
      searchResults.value = [];
      hasMore.value = false;
      return;
    }

    const startTime = performance.now();
    isSearching.value = true;
    searchError.value = null;

    try {
      // 创建新的abort controller
      searchAbortController = new AbortController();

      // 调用搜索API
      const response = await SearchService.search({
        query: query.trim(),
        chat_id: chatId,
        cursor,
        limit: PAGE_SIZE,
        signal: searchAbortController.signal
      });

      const results = response.data?.results || [];
      const newCursor = response.data?.cursor || null;

      // 更新结果
      if (cursor) {
        // 追加结果（加载更多）
        searchResults.value = [...searchResults.value, ...results];
      } else {
        // 新搜索
        searchResults.value = results;
      }

      hasMore.value = results.length === PAGE_SIZE && newCursor !== null;
      currentCursor.value = newCursor;

      // 更新性能指标
      const responseTime = performance.now() - startTime;
      performanceMetrics.value.searchCount++;
      performanceMetrics.value.avgResponseTime =
        (performanceMetrics.value.avgResponseTime * (performanceMetrics.value.searchCount - 1) + responseTime) /
        performanceMetrics.value.searchCount;
      performanceMetrics.value.lastSearchTime = responseTime;

      // 保存到搜索历史
      saveToRecentSearches(query.trim());

      // 检查响应时间
      if (responseTime > 200) {
        if (import.meta.env.DEV) {
          console.warn(`[ChatSearch] Slow search response: ${responseTime.toFixed(0)}ms`);
        }
      }

    } catch (error) {
      if (error.name === 'AbortError') {
        // 搜索被取消，忽略
        return;
      }

      if (import.meta.env.DEV) {
        console.error('[ChatSearch] Search error:', error);
      }
      searchError.value = error.message || 'Search failed';
      searchResults.value = [];
      hasMore.value = false;
    } finally {
      isSearching.value = false;
      searchAbortController = null;
    }
  };

  /**
   * Debounced搜索
   */
  const debouncedSearch = (query) => {
    // 清除之前的定时器
    if (debounceTimer) {
      clearTimeout(debounceTimer);
      performanceMetrics.value.debounceCount++;
    }

    // 立即更新查询（UI响应）
    searchQuery.value = query;

    // 如果查询为空，立即清空结果
    if (!query || query.trim().length < MIN_QUERY_LENGTH) {
      searchResults.value = [];
      hasMore.value = false;
      currentCursor.value = null;
      return;
    }

    // 设置新的定时器
    debounceTimer = setTimeout(() => {
      performSearch(query);
    }, DEBOUNCE_DELAY);
  };

  /**
   * 加载更多结果
   */
  const loadMore = async () => {
    if (!hasMore.value || isSearching.value || !currentCursor.value) {
      return;
    }

    await performSearch(searchQuery.value, currentCursor.value);
  };

  /**
   * 保存到最近搜索
   */
  const saveToRecentSearches = (query) => {
    // 移除重复项
    const filtered = recentSearches.value.filter(q => q !== query);

    // 添加到开头
    filtered.unshift(query);

    // 限制数量
    if (filtered.length > MAX_RECENT_SEARCHES) {
      filtered.length = MAX_RECENT_SEARCHES;
    }

    recentSearches.value = filtered;
    localStorage.setItem('recentSearches', JSON.stringify(filtered));
  };

  /**
   * 清空搜索
   */
  const clearSearch = () => {
    searchQuery.value = '';
    searchResults.value = [];
    hasMore.value = false;
    currentCursor.value = null;
    searchError.value = null;

    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    if (searchAbortController) {
      searchAbortController.abort();
    }
  };

  /**
   * 应用搜索预设
   */
  const applyPreset = (preset) => {
    const newQuery = searchQuery.value.trim();
    const filterRegex = /\b\w+:\w+/g;

    // 移除现有的同类过滤器
    const cleanedQuery = newQuery.replace(filterRegex, '').trim();

    // 添加新过滤器
    const finalQuery = `${cleanedQuery} ${preset.filter}`.trim();
    debouncedSearch(finalQuery);
  };

  /**
   * 高亮搜索关键词
   */
  const highlightKeyword = (text, keyword) => {
    if (!keyword || !text) return text;

    const regex = new RegExp(`(${escapeRegExp(keyword)})`, 'gi');
    return text.replace(regex, '<mark class="bg-yellow-300">$1</mark>');
  };

  /**
   * 转义正则表达式特殊字符
   */
  const escapeRegExp = (string) => {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  };

  /**
   * 获取搜索摘要
   */
  const getSearchSummary = computed(() => {
    if (!searchQuery.value) return '';
    if (isSearching.value) return 'Searching...';
    if (searchError.value) return `Error: ${searchError.value}`;
    if (searchResults.value.length === 0) return 'No results found';

    const totalText = hasMore.value ? `${searchResults.value.length}+` : searchResults.value.length;
    return `Found ${totalText} results`;
  });

  /**
   * 键盘快捷键处理 - DISABLED to prevent conflicts with PerfectSearchModal
   */
  const handleKeyboard = (event) => {
    // DISABLED: Temporarily disabled to prevent conflict with PerfectSearchModal
    // This was causing wrong search modal to appear
    /*
    // Ctrl/Cmd + K 打开搜索
    if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
      event.preventDefault();
      // 触发搜索modal打开事件
      window.dispatchEvent(new CustomEvent('open-search-modal'));
    }
    */
  };

  // 监听键盘事件 - DISABLED to prevent conflicts
  onMounted(() => {
    // DISABLED: Temporarily disabled to prevent conflict with PerfectSearchModal
    // document.addEventListener('keydown', handleKeyboard);
  });

  onUnmounted(() => {
    // DISABLED: Temporarily disabled to prevent conflict with PerfectSearchModal  
    // document.removeEventListener('keydown', handleKeyboard);

    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    if (searchAbortController) {
      searchAbortController.abort();
    }
  });

  return {
    // 状态
    searchQuery,
    searchResults,
    isSearching,
    searchError,
    hasMore,
    recentSearches,
    searchPresets,
    performanceMetrics,

    // 方法
    debouncedSearch,
    loadMore,
    clearSearch,
    applyPreset,
    highlightKeyword,

    // 计算属性
    getSearchSummary
  };
}