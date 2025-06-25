/**
 * Quick Navigation Fix
 * Direct fixes for navigation issues
 */

// Fix channel navigation
export function fixChannelNavigation() {
  // Remove all the complex router-link logic
  document.querySelectorAll('.channel-card, .dm-card').forEach(card => {
    // Remove existing click handlers
    const newCard = card.cloneNode(true);
    card.parentNode.replaceChild(newCard, card);

    // Add simple direct click handler
    newCard.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();

      // Get chat ID from the parent router-link
      const routerLink = newCard.closest('a[href*="/chat/"]');
      if (routerLink) {
        const href = routerLink.getAttribute('href');
        const chatId = href.split('/chat/')[1];

        console.log('🔧 Direct navigation to chat:', chatId);

        // Direct navigation using router
        if (window.$router) {
          window.$router.push(`/chat/${chatId}`).catch(err => {
            console.error('Navigation error:', err);
            // Fallback: direct window navigation
            window.location.href = `/chat/${chatId}`;
          });
        } else {
          // Fallback
          window.location.href = `/chat/${chatId}`;
        }
      }
    });
  });

  console.log('✅ Channel navigation fixed');
}

// Fix logout
export function fixLogout() {
  const logoutButtons = document.querySelectorAll('.logout, [class*="logout"]');

  logoutButtons.forEach(btn => {
    // Remove existing handlers
    const newBtn = btn.cloneNode(true);
    btn.parentNode.replaceChild(newBtn, btn);

    // Add direct handler
    newBtn.addEventListener('click', async (e) => {
      e.preventDefault();
      e.stopPropagation();

      console.log('🔧 Direct logout triggered');

      // Clear all auth data directly
      localStorage.clear();
      sessionStorage.clear();

      // Clear cookies
      document.cookie.split(";").forEach(c => {
        document.cookie = c.replace(/^ +/, "").replace(/=.*/, "=;expires=" + new Date().toUTCString() + ";path=/");
      });

      // Direct redirect to login
      window.location.href = '/login';
    });
  });

  console.log('✅ Logout fixed');
}

// Fix router issues
export function fixRouter() {
  // Ensure router is globally available
  if (window.$router) {
    // Override push to add error handling
    const originalPush = window.$router.push;
    window.$router.push = function (...args) {
      console.log('🔧 Router push:', args);
      return originalPush.apply(this, args).catch(err => {
        console.error('Router push error:', err);
        // Fallback to direct navigation
        if (typeof args[0] === 'string') {
          window.location.href = args[0];
        }
      });
    };
  }

  console.log('✅ Router fixed');
}

// Apply all fixes
export function applyAllFixes() {
  console.group('🔧 Applying navigation fixes');

  try {
    fixRouter();

    // Wait for DOM to be ready
    setTimeout(() => {
      fixChannelNavigation();
      fixLogout();
    }, 500);

    // Reapply fixes when DOM changes
    const observer = new MutationObserver(() => {
      fixChannelNavigation();
      fixLogout();
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true
    });

    console.log('✅ All navigation fixes applied');
  } catch (error) {
    console.error('❌ Error applying fixes:', error);
  }

  console.groupEnd();
}

// Auto-apply fixes on load
if (typeof window !== 'undefined') {
  window.addEventListener('DOMContentLoaded', () => {
    applyAllFixes();
  });

  // Expose for manual use
  window.navigationFix = {
    fixChannelNavigation,
    fixLogout,
    fixRouter,
    applyAllFixes
  };

  console.log('🔧 Navigation fix loaded. Use window.navigationFix.applyAllFixes()');
} 