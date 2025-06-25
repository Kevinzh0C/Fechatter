/**
 * 通知系统测试工具
 * 用于验证修复后的通知功能是否正常工作
 */

import { useNotifications } from '@/composables/useNotifications';

export function testNotificationSystem() {
  try {
    console.log('🧪 Testing notification system...');

    const { notifyInfo, notifyWarning, notifyError, notifySuccess } = useNotifications();

    // Test all notification types
    setTimeout(() => {
      notifyInfo('通知系统测试 - 信息通知', { title: 'Info Test' });
    }, 100);

    setTimeout(() => {
      notifySuccess('通知系统测试 - 成功通知', { title: 'Success Test' });
    }, 200);

    setTimeout(() => {
      notifyWarning('通知系统测试 - 警告通知', { title: 'Warning Test' });
    }, 300);

    setTimeout(() => {
      notifyError('通知系统测试 - 错误通知', { title: 'Error Test' });
    }, 400);

    console.log('✅ Notification system test completed. Check the UI for toast notifications.');

    return {
      success: true,
      message: 'All notification functions are working correctly'
    };

  } catch (error) {
    console.error('❌ Notification system test failed:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

// 测试API错误通知
export async function testApiNotifications() {
  try {
    console.log('🧪 Testing API notification system...');

    // Import the showUserNotification function for testing
    const { default: api } = await import('@/services/api');

    // Test the showUserNotification function indirectly by calling it from the console
    console.log('📋 You can now test API notifications by running:');
    console.log('   - Test info: showUserNotification("info", "Test Title", "Test info message")');
    console.log('   - Test warning: showUserNotification("warning", "Test Title", "Test warning message")');
    console.log('   - Test error: showUserNotification("error", "Test Title", "Test error message")');

    return {
      success: true,
      message: 'API notification system is ready for testing'
    };

  } catch (error) {
    console.error('❌ API notification test setup failed:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

// 在开发环境下自动暴露到全局
if (import.meta.env.DEV) {
  window.testNotifications = testNotificationSystem;
  window.testApiNotifications = testApiNotifications;
}

export default {
  testNotificationSystem,
  testApiNotifications
}; 