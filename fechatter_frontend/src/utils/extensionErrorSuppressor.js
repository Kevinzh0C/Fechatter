/**
 * Extension Error Suppressor
 * 专门处理浏览器扩展产生的错误
 */

class ExtensionErrorSuppressor {
  constructor() {
    this.suppressedPatterns = [
      // Chrome扩展通信错误
      /A listener indicated an asynchronous response by returning true, but the message channel closed before a response was received/,
      /Extension context invalidated/,
      /Cannot access contents of url/,
      /message channel closed before a response/,

      // QuillBot扩展特定错误
      /quillbot-content\.js/,
      /quillbot.*undefined/,

      // 其他常见扩展错误
      /content.*script.*error/i,
      /chrome-extension:/,
      /moz-extension:/
    ];

    this.initialize();
  }

  initialize() {
    // 拦截全局错误
    window.addEventListener('error', this.handleError.bind(this), true);
    window.addEventListener('unhandledrejection', this.handlePromiseRejection.bind(this), true);

    console.log('🛡️ Extension Error Suppressor initialized');
  }

  handleError(event) {
    if (this.shouldSuppress(event.error || event.message)) {
      event.stopImmediatePropagation();
      event.preventDefault();
      return false;
    }
  }

  handlePromiseRejection(event) {
    if (this.shouldSuppress(event.reason)) {
      event.stopImmediatePropagation();
      event.preventDefault();
      return false;
    }
  }

  shouldSuppress(error) {
    if (!error) return false;

    const message = error.message || error.toString() || '';
    const stack = error.stack || '';

    // 检查错误消息
    const messageMatch = this.suppressedPatterns.some(pattern => pattern.test(message));

    // 检查堆栈信息
    const stackMatch = this.suppressedPatterns.some(pattern => pattern.test(stack));

    // 检查文件来源（扩展文件）
    const isExtensionFile = stack.includes('chrome-extension://') ||
      stack.includes('moz-extension://') ||
      message.includes('chrome-extension://') ||
      message.includes('moz-extension://');

    return messageMatch || stackMatch || isExtensionFile;
  }
}

// 立即初始化
if (typeof window !== 'undefined') {
  new ExtensionErrorSuppressor();
}

export default ExtensionErrorSuppressor; 