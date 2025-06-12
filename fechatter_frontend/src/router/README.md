# Router Architecture

## 📁 文件结构

```
src/router/
├── index.ts              # 主路由配置，全局守卫
├── types.ts              # TypeScript 类型定义
├── utils.ts              # 工具函数(会话恢复、性能监控)
├── middleware/           # 中间件管道
│   ├── index.ts         # 导出所有中间件
│   ├── auth.ts          # 认证中间件
│   ├── permissions.ts   # 权限中间件
│   ├── logger.ts        # 日志中间件
│   ├── progress.ts      # 进度条中间件
│   └── title.ts         # 页面标题中间件
└── routes/              # 模块化路由定义
    ├── index.ts         # 路由汇总
    ├── auth.routes.ts   # 认证相关路由
    ├── app.routes.ts    # 主应用路由
    ├── demo.routes.ts   # 演示页面路由
    └── error.routes.ts  # 错误页面路由
```

## ✅ 规范遵循情况

### 1. 懒加载 & 代码分割 ✅
- 所有页面组件使用 `() => import()` 动态导入
- 顶层路由控制在 8 个以内
- 支持按需加载和代码分块

### 2. 模块化路由文件 ✅
- 根路由文件仅注册全局守卫
- 业务路由拆分到独立的 `.routes.ts` 文件
- 通过 `...spread` 语法合并路由

### 3. TypeScript 化 ✅
- 完整的 TypeScript 支持
- `RouteRecordRaw` 类型约束
- 扩展的 `RouteMeta` 接口

### 4. 多历史模式支持 ✅
- 默认使用 `createWebHistory(import.meta.env.BASE_URL)`
- SSR 环境自动切换到 `createMemoryHistory`
- 支持自定义 base URL

### 5. Middleware Pipeline ✅
- 可组合的中间件数组架构
- 支持中间件链式执行
- 替代了传统的多个 `beforeEach`

### 6. 进度条 & 页面标题 ✅
- 集成 `nprogress` 显示导航进度
- 根据 `meta.title` 自动更新页面标题
- 自定义样式和动画配置

### 7. 错误路由 ✅
- 统一的 `/error/:code` 组件
- 404/403/500 等错误页面
- 未匹配路由自动重定向到 404

### 8. 会话恢复 ✅
- 智能重定向系统
- 30分钟超时机制
- 登录后自动恢复原路径

### 9. Debug & 生产日志 ✅
- 仅在 `VITE_DEBUG=true` 时输出调试日志
- 自动生成 trace ID
- 性能监控和错误上报

### 10. 自动化测试钩子 ✅
- 导出 `createTestRouter()` 工厂函数
- 支持注入自定义初始路径
- 测试环境下禁用中间件

## 🚀 使用示例

### 基本使用
```ts
import router from '@/router';

// 已自动配置所有中间件和路由
app.use(router);
```

### 测试环境
```ts
import { createTestRouter } from '@/router';

const testRouter = createTestRouter('/demo');
```

### 自定义配置
```ts
import { createAppRouter } from '@/router';

const router = createAppRouter({
  baseUrl: '/app/',
  enableProgress: false,
  middleware: [customMiddleware],
});
```

## 🔧 扩展指南

### 添加新路由模块
1. 在 `routes/` 目录创建 `xxx.routes.ts`
2. 在 `routes/index.ts` 中导入并合并
3. 使用懒加载语法导入组件

### 创建自定义中间件
```ts
import type { RouterMiddleware } from '@/router/types';

export const customMiddleware: RouterMiddleware = async (to, from, next) => {
  // 中间件逻辑
  next();
};
```

### 路由元信息配置
```ts
meta: {
  title: '页面标题',
  requiresAuth: true,
  roles: ['admin'],
  permissions: ['read:users'],
  layout: 'default',
  keepAlive: true,
  hideInMenu: false,
  icon: '🏠',
  order: 1,
}
```