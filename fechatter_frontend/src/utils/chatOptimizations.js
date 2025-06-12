/**
 * Chat Optimizations
 * Reduce flicker, improve performance, and enhance user experience
 */

/**
 * Debounce function to prevent rapid calls
 */
export function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

/**
 * Throttle function to limit call frequency
 */
export function throttle(func, limit) {
  let inThrottle;
  return function (...args) {
    if (!inThrottle) {
      func.apply(this, args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, limit);
    }
  };
}

/**
 * Preload user avatars to prevent flicker
 */
export function preloadAvatars(users) {
  const avatarsToLoad = users
    .filter(user => user.avatar_url)
    .map(user => user.avatar_url);

  avatarsToLoad.forEach(url => {
    const img = new Image();
    img.src = url;
  });
}

/**
 * Apply skeleton loading styles
 */
export function applySkeletonLoading(element) {
  element.classList.add('skeleton-loading');
}

/**
 * Remove skeleton loading styles
 */
export function removeSkeletonLoading(element) {
  element.classList.remove('skeleton-loading');
}

/**
 * Virtual scrolling helper for large message lists
 */
export class VirtualScroller {
  constructor(container, itemHeight = 60) {
    this.container = container;
    this.itemHeight = itemHeight;
    this.visibleItems = Math.ceil(container.clientHeight / itemHeight) + 2;
    this.startIndex = 0;
  }

  updateVisibleRange(scrollTop) {
    this.startIndex = Math.floor(scrollTop / this.itemHeight);
    return {
      start: Math.max(0, this.startIndex - 5), // Buffer of 5 items
      end: this.startIndex + this.visibleItems + 5
    };
  }
}

/**
 * Smooth scroll to element
 */
export function smoothScrollTo(element, container, offset = 0) {
  const targetPosition = element.offsetTop - offset;
  const startPosition = container.scrollTop;
  const distance = targetPosition - startPosition;
  const duration = 300;
  let start = null;

  function animation(currentTime) {
    if (start === null) start = currentTime;
    const timeElapsed = currentTime - start;
    const progress = Math.min(timeElapsed / duration, 1);

    // Easing function
    const easeInOutQuad = progress < 0.5
      ? 2 * progress * progress
      : -1 + (4 - 2 * progress) * progress;

    container.scrollTop = startPosition + (distance * easeInOutQuad);

    if (timeElapsed < duration) {
      requestAnimationFrame(animation);
    }
  }

  requestAnimationFrame(animation);
}

/**
 * Optimize images for better performance
 */
export function optimizeImageLoading() {
  // Use Intersection Observer for lazy loading
  const imageObserver = new IntersectionObserver((entries, observer) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const img = entry.target;
        img.src = img.dataset.src;
        img.classList.remove('lazy');
        observer.unobserve(img);
      }
    });
  });

  // Observe all lazy images
  document.querySelectorAll('img.lazy').forEach(img => {
    imageObserver.observe(img);
  });

  return imageObserver;
}

/**
 * CSS for optimizations
 */
export const optimizationStyles = `
  /* Skeleton loading animation */
  .skeleton-loading {
    background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
    background-size: 200% 100%;
    animation: loading 1.5s infinite;
  }

  @keyframes loading {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  /* Prevent layout shift */
  .message-avatar {
    aspect-ratio: 1;
    contain: layout;
  }

  /* Smooth transitions */
  .message-item {
    transition: transform 0.2s ease, opacity 0.2s ease;
  }

  /* Optimize rendering */
  .message-list {
    will-change: scroll-position;
    -webkit-overflow-scrolling: touch;
  }

  /* Prevent flicker on hover */
  .message-item:hover {
    transform: translateZ(0);
    backface-visibility: hidden;
  }

  /* Lazy image placeholder */
  img.lazy {
    background-color: #f0f0f0;
    min-height: 40px;
  }
`;

/**
 * Inject optimization styles
 */
export function injectOptimizationStyles() {
  const style = document.createElement('style');
  style.textContent = optimizationStyles;
  document.head.appendChild(style);
}

// Export all optimizations
export default {
  debounce,
  throttle,
  preloadAvatars,
  applySkeletonLoading,
  removeSkeletonLoading,
  VirtualScroller,
  smoothScrollTo,
  optimizeImageLoading,
  injectOptimizationStyles
}; 