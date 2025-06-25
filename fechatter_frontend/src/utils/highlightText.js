/**
 * M3: 关键词双高亮（列表 & 原文）
 * 一致性原则；同样 mark.bg-yellow-300 token
 */

/**
 * 高亮文本中的关键词
 * @param {string} text - 原始文本
 * @param {string} keyword - 要高亮的关键词
 * @param {Object} options - 配置选项
 * @returns {string} - 带有高亮标记的HTML
 */
export function highlightKeyword(text, keyword, options = {}) {
  const {
    className = 'bg-yellow-300',
    tag = 'mark',
    caseSensitive = false,
    wholeWord = false,
    maxLength = null
  } = options;

  if (!text || !keyword) return text;

  // 转义HTML特殊字符
  const escapeHtml = (str) => {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  };

  // 转义正则表达式特殊字符
  const escapeRegExp = (str) => {
    return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  };

  // 截断文本（如果需要）
  let processedText = text;
  let ellipsisStart = '';
  let ellipsisEnd = '';

  if (maxLength && text.length > maxLength) {
    const lowerText = caseSensitive ? text : text.toLowerCase();
    const lowerKeyword = caseSensitive ? keyword : keyword.toLowerCase();
    const keywordIndex = lowerText.indexOf(lowerKeyword);

    if (keywordIndex !== -1) {
      // 关键词在文本中，围绕关键词截断
      const contextRadius = Math.floor((maxLength - keyword.length) / 2);
      const start = Math.max(0, keywordIndex - contextRadius);
      const end = Math.min(text.length, keywordIndex + keyword.length + contextRadius);

      // 调整到词边界
      let adjustedStart = start;
      let adjustedEnd = end;

      if (start > 0) {
        const spaceIndex = text.lastIndexOf(' ', start + 10);
        if (spaceIndex > start - 20) {
          adjustedStart = spaceIndex + 1;
        }
        ellipsisStart = '...';
      }

      if (end < text.length) {
        const spaceIndex = text.indexOf(' ', end - 10);
        if (spaceIndex !== -1 && spaceIndex < end + 20) {
          adjustedEnd = spaceIndex;
        }
        ellipsisEnd = '...';
      }

      processedText = text.substring(adjustedStart, adjustedEnd);
    } else {
      // 关键词不在文本中，截断开头
      processedText = text.substring(0, maxLength);
      const lastSpace = processedText.lastIndexOf(' ');
      if (lastSpace > maxLength - 20) {
        processedText = processedText.substring(0, lastSpace);
      }
      ellipsisEnd = '...';
    }
  }

  // 构建正则表达式
  const escapedKeyword = escapeRegExp(keyword);
  let pattern = escapedKeyword;
  
  if (wholeWord) {
    pattern = `\\b${pattern}\\b`;
  }

  const flags = caseSensitive ? 'g' : 'gi';
  const regex = new RegExp(`(${pattern})`, flags);

  // 转义HTML并高亮
  const escapedText = escapeHtml(processedText);
  const highlighted = escapedText.replace(regex, `<${tag} class="${className}">$1</${tag}>`);

  return ellipsisStart + highlighted + ellipsisEnd;
}

/**
 * 高亮多个关键词
 * @param {string} text - 原始文本
 * @param {string[]} keywords - 关键词数组
 * @param {Object} options - 配置选项
 * @returns {string} - 带有高亮标记的HTML
 */
export function highlightMultipleKeywords(text, keywords, options = {}) {
  if (!text || !keywords || keywords.length === 0) return text;

  // 按长度降序排序，避免短词干扰长词
  const sortedKeywords = [...keywords].sort((a, b) => b.length - a.length);
  
  let result = text;
  sortedKeywords.forEach((keyword, index) => {
    // 为每个关键词使用不同的类
    const customOptions = {
      ...options,
      className: options.classNames?.[index] || options.className || 'bg-yellow-300'
    };
    
    // 使用临时标记避免重复高亮
    const tempTag = `temp-mark-${index}`;
    result = highlightKeyword(result, keyword, { ...customOptions, tag: tempTag });
  });

  // 将临时标记转换为最终标记
  sortedKeywords.forEach((_, index) => {
    const tempTag = `temp-mark-${index}`;
    result = result.replace(new RegExp(tempTag, 'g'), options.tag || 'mark');
  });

  return result;
}

/**
 * 获取高亮片段的上下文
 * @param {string} text - 原始文本
 * @param {string} keyword - 关键词
 * @param {number} contextLength - 上下文长度
 * @returns {Array} - 包含高亮片段的数组
 */
export function getHighlightedSnippets(text, keyword, contextLength = 50) {
  if (!text || !keyword) return [];

  const lowerText = text.toLowerCase();
  const lowerKeyword = keyword.toLowerCase();
  const snippets = [];
  let startIndex = 0;

  while (startIndex < text.length) {
    const index = lowerText.indexOf(lowerKeyword, startIndex);
    if (index === -1) break;

    const start = Math.max(0, index - contextLength);
    const end = Math.min(text.length, index + keyword.length + contextLength);
    
    const snippet = {
      text: text.substring(start, end),
      start,
      end,
      matchStart: index - start,
      matchEnd: index - start + keyword.length
    };

    snippets.push(snippet);
    startIndex = index + 1;
  }

  return snippets;
}

/**
 * 计算文本相关性分数
 * @param {string} text - 原始文本
 * @param {string} keyword - 关键词
 * @returns {number} - 相关性分数 (0-1)
 */
export function calculateRelevanceScore(text, keyword) {
  if (!text || !keyword) return 0;

  const lowerText = text.toLowerCase();
  const lowerKeyword = keyword.toLowerCase();
  
  // 完全匹配
  if (lowerText === lowerKeyword) return 1;

  // 计算出现次数
  const matches = (lowerText.match(new RegExp(escapeRegExp(lowerKeyword), 'g')) || []).length;
  if (matches === 0) return 0;

  // 计算密度
  const density = (matches * keyword.length) / text.length;
  
  // 位置权重（靠前的匹配更重要）
  const firstMatchIndex = lowerText.indexOf(lowerKeyword);
  const positionScore = 1 - (firstMatchIndex / text.length);

  // 综合评分
  return Math.min(1, (density * 0.5 + positionScore * 0.5) * Math.log(matches + 1));
}

/**
 * 创建高亮正则表达式
 * @param {string} keyword - 关键词
 * @param {Object} options - 配置选项
 * @returns {RegExp} - 正则表达式
 */
function escapeRegExp(str) {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}