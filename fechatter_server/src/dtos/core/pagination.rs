// 分页和查询相关的DTOs
//
// 提供统一的分页、排序、过滤功能
// 这些DTOs可以被所有需要列表查询的API重用

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// 分页请求参数 - 所有列表查询的基础
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PaginationRequest {
  /// 页码，从1开始
  #[validate(range(min = 1, max = 10000, message = "页码必须在1-10000之间"))]
  #[serde(default = "default_page")]
  pub page: u32,

  /// 每页数量  
  #[validate(range(min = 1, max = 100, message = "每页数量必须在1-100之间"))]
  #[serde(default = "default_page_size")]
  pub page_size: u32,

  /// 排序字段和方向
  #[serde(default)]
  pub sort: Vec<SortField>,

  /// 通用过滤条件
  #[serde(default)]
  pub filters: HashMap<String, FilterValue>,

  /// 搜索关键词
  #[validate(length(max = 100, message = "搜索关键词最多100个字符"))]
  pub search: Option<String>,
}

/// 分页响应结果 - 所有列表查询的标准返回格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
  /// 数据列表
  pub data: Vec<T>,

  /// 分页元信息
  pub pagination: PaginationMeta,

  /// 查询统计信息
  pub stats: Option<QueryStats>,
}

/// 分页元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
  /// 当前页码
  pub current_page: u32,

  /// 每页数量
  pub page_size: u32,

  /// 总记录数
  pub total_items: u64,

  /// 总页数
  pub total_pages: u32,

  /// 是否有上一页
  pub has_previous: bool,

  /// 是否有下一页  
  pub has_next: bool,

  /// 上一页页码
  pub previous_page: Option<u32>,

  /// 下一页页码
  pub next_page: Option<u32>,
}

/// 排序字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortField {
  /// 字段名
  pub field: String,

  /// 排序方向
  pub direction: SortDirection,
}

/// 排序方向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
  #[serde(rename = "asc")]
  Ascending,

  #[serde(rename = "desc")]
  Descending,
}

/// 过滤值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
  String(String),
  Number(f64),
  Boolean(bool),
  Array(Vec<String>),
  Range {
    min: Option<f64>,
    max: Option<f64>,
  },
  DateRange {
    start: Option<chrono::DateTime<chrono::Utc>>,
    end: Option<chrono::DateTime<chrono::Utc>>,
  },
}

/// 查询统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
  /// 查询执行时间(毫秒)
  pub execution_time_ms: u64,

  /// 是否使用了缓存
  pub cached: bool,

  /// 查询复杂度评分
  pub complexity_score: Option<u32>,

  /// 相关性得分(搜索查询)
  pub relevance_scores: Option<Vec<f32>>,
}

/// 游标分页请求 - 适用于大数据量场景
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CursorPaginationRequest {
  /// 游标位置
  pub cursor: Option<String>,

  /// 每页数量
  #[validate(range(min = 1, max = 100, message = "每页数量必须在1-100之间"))]
  #[serde(default = "default_page_size")]
  pub limit: u32,

  /// 查询方向
  #[serde(default)]
  pub direction: CursorDirection,
}

/// 游标分页响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPaginatedResponse<T> {
  /// 数据列表
  pub data: Vec<T>,

  /// 下一页游标
  pub next_cursor: Option<String>,

  /// 上一页游标
  pub previous_cursor: Option<String>,

  /// 是否还有更多数据
  pub has_more: bool,

  /// 查询统计
  pub stats: Option<QueryStats>,
}

/// 游标查询方向
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CursorDirection {
  #[default]
  Forward,
  Backward,
}

// 默认值函数
fn default_page() -> u32 {
  1
}
fn default_page_size() -> u32 {
  20
}

// 实用工具函数
impl PaginationRequest {
  /// 创建新的分页请求
  pub fn new(page: u32, page_size: u32) -> Self {
    Self {
      page,
      page_size,
      sort: Vec::new(),
      filters: HashMap::new(),
      search: None,
    }
  }

  /// 添加排序字段
  pub fn with_sort(mut self, field: String, direction: SortDirection) -> Self {
    self.sort.push(SortField { field, direction });
    self
  }

  /// 添加过滤条件
  pub fn with_filter(mut self, key: String, value: FilterValue) -> Self {
    self.filters.insert(key, value);
    self
  }

  /// 添加搜索关键词
  pub fn with_search(mut self, search: String) -> Self {
    self.search = Some(search);
    self
  }

  /// 计算偏移量
  pub fn offset(&self) -> u32 {
    (self.page - 1) * self.page_size
  }

  /// 计算限制数量
  pub fn limit(&self) -> u32 {
    self.page_size
  }
}

impl<T> PaginatedResponse<T> {
  /// 创建分页响应
  pub fn new(data: Vec<T>, page: u32, page_size: u32, total_items: u64) -> Self {
    let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;
    let has_previous = page > 1;
    let has_next = page < total_pages;
    let previous_page = if has_previous { Some(page - 1) } else { None };
    let next_page = if has_next { Some(page + 1) } else { None };

    Self {
      data,
      pagination: PaginationMeta {
        current_page: page,
        page_size,
        total_items,
        total_pages,
        has_previous,
        has_next,
        previous_page,
        next_page,
      },
      stats: None,
    }
  }

  /// 添加查询统计信息
  pub fn with_stats(mut self, stats: QueryStats) -> Self {
    self.stats = Some(stats);
    self
  }

  /// 转换数据类型
  pub fn map<U, F>(self, f: F) -> PaginatedResponse<U>
  where
    F: FnMut(T) -> U,
  {
    PaginatedResponse {
      data: self.data.into_iter().map(f).collect(),
      pagination: self.pagination,
      stats: self.stats,
    }
  }
}
