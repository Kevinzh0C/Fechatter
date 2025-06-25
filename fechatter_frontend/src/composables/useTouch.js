import { ref, onMounted, onUnmounted } from 'vue';

export function useTouch(elementRef) {
  const touchStartX = ref(0);
  const touchStartY = ref(0);
  const touchEndX = ref(0);
  const touchEndY = ref(0);
  const isSwiping = ref(false);
  
  // Callback storage
  const callbacks = ref({
    swipeUp: null,
    swipeDown: null,
    swipeLeft: null,
    swipeRight: null
  });

  const handleTouchStart = (e) => {
    touchStartX.value = e.touches[0].clientX;
    touchStartY.value = e.touches[0].clientY;
    isSwiping.value = true;
  };

  const handleTouchMove = (e) => {
    if (!isSwiping.value) return;
    touchEndX.value = e.touches[0].clientX;
    touchEndY.value = e.touches[0].clientY;
  };

  const handleTouchEnd = () => {
    if (!isSwiping.value) return;
    
    const deltaX = touchEndX.value - touchStartX.value;
    const deltaY = touchEndY.value - touchStartY.value;
    const minSwipeDistance = 50;
    
    // Check if it's a horizontal swipe
    if (Math.abs(deltaX) > Math.abs(deltaY) && Math.abs(deltaX) > minSwipeDistance) {
      if (deltaX > 0) {
        // Swipe right
        if (callbacks.value.swipeRight) callbacks.value.swipeRight();
      } else {
        // Swipe left
        if (callbacks.value.swipeLeft) callbacks.value.swipeLeft();
      }
    }
    
    // Check if it's a vertical swipe
    if (Math.abs(deltaY) > Math.abs(deltaX) && Math.abs(deltaY) > minSwipeDistance) {
      if (deltaY > 0) {
        // Swipe down
        if (callbacks.value.swipeDown) callbacks.value.swipeDown();
      } else {
        // Swipe up
        if (callbacks.value.swipeUp) callbacks.value.swipeUp();
      }
    }
    
    isSwiping.value = false;
  };

  const addTouchListeners = (element) => {
    if (!element) return;
    element.addEventListener('touchstart', handleTouchStart, { passive: true });
    element.addEventListener('touchmove', handleTouchMove, { passive: true });
    element.addEventListener('touchend', handleTouchEnd, { passive: true });
  };

  const removeTouchListeners = (element) => {
    if (!element) return;
    element.removeEventListener('touchstart', handleTouchStart);
    element.removeEventListener('touchmove', handleTouchMove);
    element.removeEventListener('touchend', handleTouchEnd);
  };

  // Callback registration functions
  const onSwipeUp = (callback) => {
    callbacks.value.swipeUp = callback;
  };

  const onSwipeDown = (callback) => {
    callbacks.value.swipeDown = callback;
  };

  const onSwipeLeft = (callback) => {
    callbacks.value.swipeLeft = callback;
  };

  const onSwipeRight = (callback) => {
    callbacks.value.swipeRight = callback;
  };

  // Auto-setup touch listeners if elementRef is provided
  onMounted(() => {
    if (elementRef && elementRef.value) {
      addTouchListeners(elementRef.value);
    }
  });

  onUnmounted(() => {
    if (elementRef && elementRef.value) {
      removeTouchListeners(elementRef.value);
    }
  });

  return {
    touchStartX,
    touchStartY,
    touchEndX,
    touchEndY,
    isSwiping,
    handleTouchStart,
    handleTouchMove,
    handleTouchEnd,
    addTouchListeners,
    removeTouchListeners,
    onSwipeUp,
    onSwipeDown,
    onSwipeLeft,
    onSwipeRight
  };
} 