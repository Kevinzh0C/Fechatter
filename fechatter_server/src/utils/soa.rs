//! Struct of Arrays (SoA) - 数据导向设计
//!
//! 工业界级别的内存布局优化，提升缓存命中率

use std::alloc::{alloc, dealloc, Layout};
use std::mem;
use std::ptr::NonNull;
use std::slice;

/// 传统的 AoS (Array of Structs) 消息存储
/// 这是大多数应用的默认方式，但缓存不友好
#[derive(Debug, Clone)]
pub struct MessageAoS {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub created_at: i64,
    pub content: String,
    pub is_read: bool,
    pub is_deleted: bool,
}

/// SoA (Struct of Arrays) 消息存储
/// 将相同字段存储在连续内存中，极大提升缓存命中率
pub struct MessagesSoA {
    // 热数据字段（频繁访问）
    ids: Vec<i64>,
    chat_ids: Vec<i64>,
    sender_ids: Vec<i64>,
    created_ats: Vec<i64>,

    // 冷数据字段（较少访问）
    contents: Vec<String>,
    is_reads: Vec<bool>,
    is_deleteds: Vec<bool>,

    // 容量管理
    capacity: usize,
    len: usize,
}

impl MessagesSoA {
    /// 创建指定容量的 SoA 存储
    pub fn with_capacity(capacity: usize) -> Self {
        MessagesSoA {
            ids: Vec::with_capacity(capacity),
            chat_ids: Vec::with_capacity(capacity),
            sender_ids: Vec::with_capacity(capacity),
            created_ats: Vec::with_capacity(capacity),
            contents: Vec::with_capacity(capacity),
            is_reads: Vec::with_capacity(capacity),
            is_deleteds: Vec::with_capacity(capacity),
            capacity,
            len: 0,
        }
    }

    /// 添加消息
    pub fn push(&mut self, msg: MessageAoS) {
        self.ids.push(msg.id);
        self.chat_ids.push(msg.chat_id);
        self.sender_ids.push(msg.sender_id);
        self.created_ats.push(msg.created_at);
        self.contents.push(msg.content);
        self.is_reads.push(msg.is_read);
        self.is_deleteds.push(msg.is_deleted);
        self.len += 1;
    }

    /// 批量添加消息
    pub fn extend(&mut self, messages: impl IntoIterator<Item = MessageAoS>) {
        for msg in messages {
            self.push(msg);
        }
    }

    /// 获取消息数量
    pub fn len(&self) -> usize {
        self.len
    }

    /// 批量过滤未读消息（缓存友好）
    pub fn filter_unread(&self) -> Vec<usize> {
        let mut indices = Vec::new();

        // 线性扫描 is_reads 数组，CPU 预取器会自动优化
        for (i, &is_read) in self.is_reads.iter().enumerate() {
            if !is_read {
                indices.push(i);
            }
        }

        indices
    }

    /// 批量过滤指定聊天的消息（缓存友好）
    pub fn filter_by_chat(&self, chat_id: i64) -> Vec<usize> {
        let mut indices = Vec::new();

        // 线性扫描 chat_ids 数组
        for (i, &cid) in self.chat_ids.iter().enumerate() {
            if cid == chat_id {
                indices.push(i);
            }
        }

        indices
    }

    /// 批量标记已读（缓存友好）
    pub fn mark_as_read(&mut self, indices: &[usize]) {
        for &idx in indices {
            if idx < self.len {
                self.is_reads[idx] = true;
            }
        }
    }

    /// 获取单个消息（需要时才组装）
    pub fn get(&self, index: usize) -> Option<MessageAoS> {
        if index >= self.len {
            return None;
        }

        Some(MessageAoS {
            id: self.ids[index],
            chat_id: self.chat_ids[index],
            sender_id: self.sender_ids[index],
            created_at: self.created_ats[index],
            content: self.contents[index].clone(),
            is_read: self.is_reads[index],
            is_deleted: self.is_deleteds[index],
        })
    }

    /// 迭代器访问
    pub fn iter(&self) -> MessagesSoAIter {
        MessagesSoAIter {
            messages: self,
            index: 0,
        }
    }
}

/// SoA 迭代器
pub struct MessagesSoAIter<'a> {
    messages: &'a MessagesSoA,
    index: usize,
}

impl<'a> Iterator for MessagesSoAIter<'a> {
    type Item = MessageAoS;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.messages.len {
            let msg = self.messages.get(self.index)?;
            self.index += 1;
            Some(msg)
        } else {
            None
        }
    }
}

/// 高级 SoA：使用自定义内存布局
/// 这是更极致的优化，直接控制内存布局
pub struct OptimizedMessagesSoA {
    // 单个内存块存储所有数据
    data: NonNull<u8>,
    layout: Layout,

    // 各字段的偏移量
    ids_offset: usize,
    chat_ids_offset: usize,
    sender_ids_offset: usize,
    created_ats_offset: usize,
    flags_offset: usize, // 压缩的布尔标志

    capacity: usize,
    len: usize,
}

impl OptimizedMessagesSoA {
    /// 创建优化的 SoA 存储
    ///
    /// # Safety
    /// 使用 unsafe 直接管理内存
    pub fn with_capacity(capacity: usize) -> Self {
        // 计算内存布局
        let id_size = mem::size_of::<i64>() * capacity;
        let flags_size = capacity; // 每个消息1字节的标志

        let total_size = id_size * 4 + flags_size; // 4个i64字段 + 标志
        let layout = Layout::from_size_align(total_size, 64).unwrap(); // 64字节对齐

        let data = unsafe {
            let ptr = alloc(layout);
            NonNull::new(ptr).expect("allocation failed")
        };

        OptimizedMessagesSoA {
            data,
            layout,
            ids_offset: 0,
            chat_ids_offset: id_size,
            sender_ids_offset: id_size * 2,
            created_ats_offset: id_size * 3,
            flags_offset: id_size * 4,
            capacity,
            len: 0,
        }
    }

    /// 添加消息（优化版本）
    pub fn push(
        &mut self,
        id: i64,
        chat_id: i64,
        sender_id: i64,
        created_at: i64,
        is_read: bool,
        is_deleted: bool,
    ) {
        if self.len >= self.capacity {
            panic!("OptimizedMessagesSoA is full");
        }

        unsafe {
            let base = self.data.as_ptr();
            let idx = self.len;

            // 写入各字段
            let ids = base.add(self.ids_offset) as *mut i64;
            let chat_ids = base.add(self.chat_ids_offset) as *mut i64;
            let sender_ids = base.add(self.sender_ids_offset) as *mut i64;
            let created_ats = base.add(self.created_ats_offset) as *mut i64;
            let flags = base.add(self.flags_offset) as *mut u8;

            *ids.add(idx) = id;
            *chat_ids.add(idx) = chat_id;
            *sender_ids.add(idx) = sender_id;
            *created_ats.add(idx) = created_at;

            // 压缩布尔标志到单个字节
            let flag_byte = (is_read as u8) | ((is_deleted as u8) << 1);
            *flags.add(idx) = flag_byte;
        }

        self.len += 1;
    }

    /// 批量扫描未读消息（SIMD 优化版本）
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn filter_unread_simd(&self) -> Vec<usize> {
        use std::arch::x86_64::*;

        let mut indices = Vec::new();
        let flags = self.data.as_ptr().add(self.flags_offset);

        // 使用 SIMD 一次处理 16 个标志
        let chunks = self.len / 16;
        for chunk in 0..chunks {
            let offset = chunk * 16;
            let flags_vec = _mm_loadu_si128(flags.add(offset) as *const __m128i);

            // 创建掩码：检查 is_read 位（最低位）
            let mask = _mm_set1_epi8(0x01);
            let is_read_vec = _mm_and_si128(flags_vec, mask);
            let zero = _mm_setzero_si128();
            let unread_mask = _mm_cmpeq_epi8(is_read_vec, zero);

            // 提取结果
            let mask_bits = _mm_movemask_epi8(unread_mask);
            for i in 0..16 {
                if mask_bits & (1 << i) != 0 {
                    indices.push(offset + i);
                }
            }
        }

        // 处理剩余的元素
        for i in (chunks * 16)..self.len {
            if *flags.add(i) & 0x01 == 0 {
                indices.push(i);
            }
        }

        indices
    }

    /// 获取聊天ID切片（零拷贝）
    pub fn chat_ids_slice(&self) -> &[i64] {
        unsafe {
            let ptr = self.data.as_ptr().add(self.chat_ids_offset) as *const i64;
            slice::from_raw_parts(ptr, self.len)
        }
    }

    /// 获取ID切片（零拷贝）
    pub fn ids_slice(&self) -> &[i64] {
        unsafe {
            let ptr = self.data.as_ptr().add(self.ids_offset) as *const i64;
            slice::from_raw_parts(ptr, self.len)
        }
    }
}

impl Drop for OptimizedMessagesSoA {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data.as_ptr(), self.layout);
        }
    }
}

/// 性能测试辅助函数
pub fn benchmark_aos_vs_soa(message_count: usize) {
    use std::time::Instant;

    // 生成测试数据
    let messages: Vec<MessageAoS> = (0..message_count)
        .map(|i| MessageAoS {
            id: i as i64,
            chat_id: (i % 100) as i64,
            sender_id: (i % 10) as i64,
            created_at: 1000000 + i as i64,
            content: format!("Message {}", i),
            is_read: i % 3 == 0,
            is_deleted: false,
        })
        .collect();

    // 测试 AoS 过滤未读消息
    let aos_messages = messages.clone();
    let start = Instant::now();
    let mut unread_aos = 0;
    for msg in &aos_messages {
        if !msg.is_read {
            unread_aos += 1;
        }
    }
    let aos_time = start.elapsed();

    // 测试 SoA 过滤未读消息
    let mut soa_messages = MessagesSoA::with_capacity(message_count);
    soa_messages.extend(messages);

    let start = Instant::now();
    let unread_indices = soa_messages.filter_unread();
    let soa_time = start.elapsed();

    println!(
        "AoS filter time: {:?}, found {} unread",
        aos_time, unread_aos
    );
    println!(
        "SoA filter time: {:?}, found {} unread",
        soa_time,
        unread_indices.len()
    );
    println!(
        "SoA speedup: {:.2}x",
        aos_time.as_nanos() as f64 / soa_time.as_nanos() as f64
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soa_basic() {
        let mut messages = MessagesSoA::with_capacity(10);

        messages.push(MessageAoS {
            id: 1,
            chat_id: 100,
            sender_id: 200,
            created_at: 1000,
            content: "Hello".to_string(),
            is_read: false,
            is_deleted: false,
        });

        messages.push(MessageAoS {
            id: 2,
            chat_id: 100,
            sender_id: 201,
            created_at: 2000,
            content: "World".to_string(),
            is_read: true,
            is_deleted: false,
        });

        assert_eq!(messages.len(), 2);

        let unread = messages.filter_unread();
        assert_eq!(unread, vec![0]);

        let chat_messages = messages.filter_by_chat(100);
        assert_eq!(chat_messages.len(), 2);
    }

    #[test]
    fn test_optimized_soa() {
        let mut messages = OptimizedMessagesSoA::with_capacity(100);

        messages.push(1, 100, 200, 1000, false, false);
        messages.push(2, 100, 201, 2000, true, false);
        messages.push(3, 101, 202, 3000, false, true);

        let chat_ids = messages.chat_ids_slice();
        assert_eq!(chat_ids[0], 100);
        assert_eq!(chat_ids[1], 100);
        assert_eq!(chat_ids[2], 101);
    }

    #[test]
    fn test_performance_comparison() {
        benchmark_aos_vs_soa(10000);
    }
}
