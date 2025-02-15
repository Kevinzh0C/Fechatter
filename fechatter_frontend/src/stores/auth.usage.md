# Auth Store 使用指南

## 概述

生产级别的认证状态管理，提供完整的用户认证功能，包括：
- 用户登录/注册/登出
- 令牌管理和自动刷新
- 会话持久化（记住我功能）
- 会话超时检测
- 完善的错误处理

## 基本使用

### 在组件中使用

```vue
<script setup>
import { useAuthStore } from '@/stores/auth';

const authStore = useAuthStore();

// 登录
async function handleLogin() {
  const success = await authStore.login(email, password, rememberMe);
  if (success) {
    // 登录成功
    console.log('Welcome', authStore.user.fullname);
  } else {
    // 显示错误
    console.error(authStore.error);
  }
}

// 登出
async function handleLogout() {
  await authStore.logout();
}
</script>

<template>
  <div v-if="authStore.isLoggedIn">
    <p>Welcome, {{ authStore.userDisplayName }}</p>
    <p>Your email: {{ authStore.user.email }}</p>
    <button @click="handleLogout">Logout</button>
  </div>
</template>
```

### 在路由守卫中使用

```js
router.beforeEach(async (to, from, next) => {
  const authStore = useAuthStore();
  
  // 等待初始化
  if (!authStore.isInitialized) {
    await authStore.initialize();
  }
  
  if (to.meta.requiresAuth && !authStore.isLoggedIn) {
    next('/login');
  } else {
    next();
  }
});
```

## API 参考

### State

- `user`: 当前用户信息
- `tokens`: 认证令牌对象
- `workspace`: 当前工作空间
- `isAuthenticated`: 是否已认证
- `loading`: 加载状态
- `error`: 错误信息

### Getters

- `isLoggedIn`: 是否已登录（综合检查）
- `isTokenExpired`: 令牌是否过期
- `tokenExpiresIn`: 令牌剩余有效时间（秒）
- `userDisplayName`: 用户显示名称
- `userInitials`: 用户名称缩写
- `shouldRefreshToken`: 是否需要刷新令牌

### Actions

#### `login(email, password, rememberMe?)`
用户登录

```js
const success = await authStore.login('user@example.com', 'password', true);
```

#### `register(userData)`
用户注册

```js
const success = await authStore.register({
  fullname: 'John Doe',
  email: 'john@example.com',
  password: 'password',
  confirmPassword: 'password'
});
```

#### `logout(logoutAll?)`
登出当前设备或所有设备

```js
await authStore.logout(); // 仅当前设备
await authStore.logout(true); // 所有设备
```

#### `updateProfile(updates)`
更新用户信息

```js
await authStore.updateProfile({
  fullname: 'New Name',
  bio: 'Updated bio'
});
```

#### `changePassword(currentPassword, newPassword)`
修改密码

```js
const success = await authStore.changePassword('oldpass', 'newpass');
```

## 高级功能

### 令牌自动刷新

Auth Store会在令牌过期前5分钟自动刷新：

```js
// 手动检查是否需要刷新
if (authStore.shouldRefreshToken) {
  await authStore.refreshAccessToken();
}
```

### 会话超时

检测用户不活动并自动登出：

```js
// 检查是否超时（默认30分钟）
if (authStore.checkSessionTimeout(30)) {
  await authStore.logout();
}
```

### 持久化存储

根据"记住我"选项，认证状态会保存到localStorage或sessionStorage：

```js
// 登录时启用记住我
await authStore.login(email, password, true);

// 手动持久化当前状态
authStore.persistAuthState();

// 清除持久化状态
authStore.clearPersistedState();
```

## 错误处理

所有错误都会被格式化为用户友好的消息：

```js
try {
  await authStore.login(email, password);
} catch (error) {
  // error.code: 错误代码
  // error.userMessage: 用户友好的错误消息
  // error.validationErrors: 字段验证错误（如果有）
  
  console.error(error.userMessage);
  
  if (error.validationErrors) {
    // 显示字段错误
    Object.entries(error.validationErrors).forEach(([field, errors]) => {
      console.error(`${field}: ${errors.join(', ')}`);
    });
  }
}
```

## 最佳实践

1. **初始化**: 在应用启动时初始化auth store
2. **错误处理**: 始终处理认证操作的错误
3. **加载状态**: 使用loading状态显示加载指示器
4. **会话管理**: 定期检查会话超时
5. **安全退出**: 敏感操作后考虑登出所有设备

## 调试

开启调试日志：

```js
// 在开发环境自动开启
if (import.meta.env.DEV) {
  console.log('Auth state:', authStore.$state);
  console.log('Is logged in:', authStore.isLoggedIn);
  console.log('Token expires in:', authStore.tokenExpiresIn, 'seconds');
}
```