# 性能测量使用指南

## 如何测量前端和后端耗时

### 1. 实时查看单条消息性能

发送消息后，在浏览器控制台会看到详细的性能数据：

```
⏱️ [PERF] Frontend processing time: 2.35ms
🌐 [PERF] Starting backend request...
⏱️ [PERF] Backend response time: 8234.67ms
📊 [PERF] Message Send Performance Summary:
  ├─ Frontend (UI Update): 2.35ms
  ├─ Backend (API Call): 8234.67ms
  ├─ Post-processing: 5.12ms
  └─ Total Time: 8241.14ms
⚠️ [PERF] Slow backend detected: 8234.67ms
```

### 2. 使用性能监控工具

在浏览器控制台输入以下命令：

#### 查看性能报告
```javascript
window.msgPerf.report()
```

输出示例：
```
📊 Message Send Performance Report
==================================
Total Messages: 10
Success Rate: 100.0%

Average Times:
  Frontend: 2.45ms
  Backend: 6543.21ms
  Total: 6548.89ms

Backend Performance:
  Min: 234.56ms
  Max: 8234.67ms
  Slow Requests (>3s): 8 (80.0%)

Recent Messages:
  1. [3:28:05 PM] Frontend: 2ms, Backend: 8235ms ⚠️ SLOW
  2. [3:29:12 PM] Frontend: 3ms, Backend: 5432ms ⚠️ SLOW
  3. [3:30:45 PM] Frontend: 2ms, Backend: 234ms
```

#### 获取统计数据
```javascript
window.msgPerf.stats()
```

返回对象：
```javascript
{
  count: 10,
  successCount: 10,
  failureCount: 0,
  avgFrontendTime: 2.45,
  avgBackendTime: 6543.21,
  avgTotalTime: 6548.89,
  minBackendTime: 234.56,
  maxBackendTime: 8234.67,
  slowBackendCount: 8,
  slowBackendPercentage: "80.0"
}
```

#### 查看最近的消息
```javascript
window.msgPerf.recent(5)  // 最近5条
```

#### 查看慢请求
```javascript
window.msgPerf.slow()  // 所有超过3秒的请求
```

#### 清空统计数据
```javascript
window.msgPerf.clear()
```

### 3. 性能指标解释

- **Frontend (UI Update)**: 前端处理时间，包括创建优化消息、更新UI等
  - 正常范围：1-5ms
  - 优秀：<3ms

- **Backend (API Call)**: 后端API响应时间
  - 优秀：<500ms
  - 良好：500ms-1s
  - 一般：1s-3s
  - 慢：>3s

- **Post-processing**: 后处理时间，包括替换优化消息、更新缓存等
  - 正常范围：1-10ms

### 4. 性能优化建议

根据测量结果：

1. **如果前端慢（>10ms）**：
   - 检查是否有大量DOM操作
   - 优化消息渲染逻辑

2. **如果后端慢（>3s）**：
   - 检查网络连接
   - 考虑使用更近的服务器
   - 优化后端处理逻辑
   - 使用消息队列异步处理

3. **如果后处理慢（>20ms）**：
   - 检查缓存更新逻辑
   - 优化消息替换算法

### 5. 导出性能数据

可以将性能数据导出用于分析：

```javascript
// 获取所有性能数据
const perfData = {
  stats: window.msgPerf.stats(),
  recent: window.msgPerf.recent(50),
  slow: window.msgPerf.slow()
};

// 导出为JSON
console.log(JSON.stringify(perfData, null, 2));
```

### 6. 实际案例

从你的日志看，European Market Channel 的消息发送：
- 前端处理：约2-3ms（优秀）
- 后端响应：8234ms（非常慢）
- 总耗时：约8秒

**结论**：前端性能优秀，瓶颈在后端响应时间。 