import { ref, readonly, onBeforeUnmount } from 'vue';

interface LoginGuardOptions {
  message?: string;
}

export function useLoginGuard(options: LoginGuardOptions = {}) {
  const isLoginInProgress = ref<boolean>(false);
  const defaultMessage = 'Login is in progress. Are you sure you want to leave?';
  const message = options.message || defaultMessage;

  // Prevent page refresh/navigation during login
  const handleBeforeUnload = (event: BeforeUnloadEvent) => {
    if (isLoginInProgress.value) {
      event.preventDefault();
      event.returnValue = message;
      return message;
    }
  };

  const startLoginGuard = (): void => {
    isLoginInProgress.value = true;
    window.addEventListener('beforeunload', handleBeforeUnload);
  };

  const stopLoginGuard = (): void => {
    isLoginInProgress.value = false;
    window.removeEventListener('beforeunload', handleBeforeUnload);
  };

  // Cleanup on component unmount
  onBeforeUnmount(() => {
    stopLoginGuard();
  });

  return {
    isLoginInProgress: readonly(isLoginInProgress),
    startLoginGuard,
    stopLoginGuard,
  };
}