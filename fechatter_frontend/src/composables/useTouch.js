import { ref, onMounted, onUnmounted } from 'vue';

export function useTouch() {
  const touchStartX = ref(0);
  const touchStartY = ref(0);
  const touchEndX = ref(0);
  const touchEndY = ref(0);
  const isSwiping = ref(false);

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
        return 'swipe-right';
      } else {
        // Swipe left
        return 'swipe-left';
      }
    }
    
    // Check if it's a vertical swipe
    if (Math.abs(deltaY) > Math.abs(deltaX) && Math.abs(deltaY) > minSwipeDistance) {
      if (deltaY > 0) {
        // Swipe down
        return 'swipe-down';
      } else {
        // Swipe up
        return 'swipe-up';
      }
    }
    
    isSwiping.value = false;
    return null;
  };

  const addTouchListeners = (element) => {
    element.addEventListener('touchstart', handleTouchStart, { passive: true });
    element.addEventListener('touchmove', handleTouchMove, { passive: true });
    element.addEventListener('touchend', handleTouchEnd, { passive: true });
  };

  const removeTouchListeners = (element) => {
    element.removeEventListener('touchstart', handleTouchStart);
    element.removeEventListener('touchmove', handleTouchMove);
    element.removeEventListener('touchend', handleTouchEnd);
  };

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
    removeTouchListeners
  };
} 