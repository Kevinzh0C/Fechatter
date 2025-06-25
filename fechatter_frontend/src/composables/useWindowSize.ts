import { ref, readonly, onMounted, onUnmounted } from 'vue';

export interface WindowSize {
  width: number;
  height: number;
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
}

export function useWindowSize() {
  const width = ref<number>(0);
  const height = ref<number>(0);

  const updateSize = (): void => {
    if (typeof window !== 'undefined') {
      width.value = window.innerWidth;
      height.value = window.innerHeight;
    }
  };

  const isMobile = ref<boolean>(false);
  const isTablet = ref<boolean>(false);
  const isDesktop = ref<boolean>(false);

  const updateBreakpoints = (): void => {
    const w = width.value;
    isMobile.value = w < 768;
    isTablet.value = w >= 768 && w < 1024;
    isDesktop.value = w >= 1024;
  };

  const handleResize = (): void => {
    updateSize();
    updateBreakpoints();
  };

  onMounted(() => {
    updateSize();
    updateBreakpoints();
    window.addEventListener('resize', handleResize, { passive: true });
  });

  onUnmounted(() => {
    window.removeEventListener('resize', handleResize);
  });

  return {
    width: readonly(width),
    height: readonly(height),
    isMobile: readonly(isMobile),
    isTablet: readonly(isTablet),
    isDesktop: readonly(isDesktop),
    windowSize: readonly({
      width,
      height,
      isMobile,
      isTablet,
      isDesktop,
    } as WindowSize),
  };
}