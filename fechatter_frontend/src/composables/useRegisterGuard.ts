import { ref, readonly, onBeforeUnmount } from 'vue';

interface RegisterGuardOptions {
  message?: string;
}

export function useRegisterGuard(options: RegisterGuardOptions = {}) {
  const isRegistrationInProgress = ref<boolean>(false);
  const defaultMessage = 'Registration is in progress. Are you sure you want to leave?';
  const message = options.message || defaultMessage;

  // Prevent page refresh/navigation during registration
  const handleBeforeUnload = (event: BeforeUnloadEvent) => {
    if (isRegistrationInProgress.value) {
      event.preventDefault();
      event.returnValue = message;
      return message;
    }
  };

  const startRegisterGuard = (): void => {
    isRegistrationInProgress.value = true;
    window.addEventListener('beforeunload', handleBeforeUnload);
  };

  const stopRegisterGuard = (): void => {
    isRegistrationInProgress.value = false;
    window.removeEventListener('beforeunload', handleBeforeUnload);
  };

  // Cleanup on component unmount
  onBeforeUnmount(() => {
    stopRegisterGuard();
  });

  return {
    isRegistrationInProgress: readonly(isRegistrationInProgress),
    startRegisterGuard,
    stopRegisterGuard,
  };
}