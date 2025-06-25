import { ref, reactive, readonly } from 'vue';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message: string;
  duration?: number;
  timestamp: number;
}

export interface ToastOptions {
  type?: ToastType;
  title?: string;
  duration?: number;
}

const toasts = ref<Toast[]>([]);
let toastIdCounter = 0;

export function useToast() {
  const addToast = (message: string, options: ToastOptions = {}): string => {
    const id = `toast-${++toastIdCounter}`;
    const toast: Toast = {
      id,
      type: options.type || 'info',
      title: options.title || '',
      message,
      duration: options.duration ?? 5000,
      timestamp: Date.now(),
    };

    toasts.value.unshift(toast);

    // Auto-remove after duration
    if (toast.duration > 0) {
      setTimeout(() => {
        removeToast(id);
      }, toast.duration);
    }

    return id;
  };

  const removeToast = (id: string): void => {
    const index = toasts.value.findIndex(toast => toast.id === id);
    if (index > -1) {
      toasts.value.splice(index, 1);
    }
  };

  const clearAllToasts = (): void => {
    toasts.value = [];
  };

  // Convenience methods
  const success = (message: string, options: Omit<ToastOptions, 'type'> = {}): string => {
    return addToast(message, { ...options, type: 'success' });
  };

  const error = (message: string, options: Omit<ToastOptions, 'type'> = {}): string => {
    return addToast(message, { 
      ...options, 
      type: 'error',
      duration: options.duration ?? 8000, // Errors stay longer
    });
  };

  const warning = (message: string, options: Omit<ToastOptions, 'type'> = {}): string => {
    return addToast(message, { ...options, type: 'warning' });
  };

  const info = (message: string, options: Omit<ToastOptions, 'type'> = {}): string => {
    return addToast(message, { ...options, type: 'info' });
  };

  return {
    toasts: readonly(toasts),
    addToast,
    removeToast,
    clearAllToasts,
    success,
    error,
    warning,
    info,
  };
}