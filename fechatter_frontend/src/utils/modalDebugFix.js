/**
 * Developer Modal Debug & Fix Utility
 * Comprehensive tool for diagnosing and fixing modal display issues
 */

class ModalDebugFix {
  constructor() {
    this.initialized = false;
    this.checks = [];
    this.fixes = [];
  }

  init() {
    if (this.initialized) return;
    this.initialized = true;
    if (import.meta.env.DEV) {
      console.log('Modal Debug Fix initialized');
    }

  // Comprehensive modal state diagnosis
  diagnose() {
    if (import.meta.env.DEV) {
      console.log('Modal Diagnosis Starting...');
    }

    const diagnosis = {
      timestamp: new Date().toISOString(),
      environment: import.meta.env.DEV ? 'development' : 'production',
      checks: []
    };

    // Check 1: Vue reactive state
    const vueState = this.checkVueState();
    diagnosis.checks.push(vueState);

    // Check 2: DOM elements
    const domState = this.checkDOMState();
    diagnosis.checks.push(domState);

    // Check 3: CSS styles
    const cssState = this.checkCSSState();
    diagnosis.checks.push(cssState);

    // Check 4: LocalStorage state
    const storageState = this.checkStorageState();
    diagnosis.checks.push(storageState);

    // Check 5: Event handlers
    const eventsState = this.checkEventHandlers();
    diagnosis.checks.push(eventsState);

    // Check 6: Extension interference
    const extensionState = this.checkExtensionInterference();
    diagnosis.checks.push(extensionState);

    this.printDiagnosis(diagnosis);
    return diagnosis;
  }

  checkVueState() {
    const check = {
      name: 'Vue Reactive State',
      status: 'checking',
      details: {}
    };

    try {
      // Try to access Vue app instance
      const app = document.querySelector('#app').__vue_app__;
      if (app) {
        check.details.vueAppFound = true;
      }

      // Check global functions
      check.details.globalFunctions = {
        showDevModal: typeof window.showDevModal === 'function',
        hideDevModal: typeof window.hideDevModal === 'function',
        resetDevModal: typeof window.resetDevModal === 'function',
        debugModal: typeof window.debugModal === 'function'
      };

      check.status = 'success';
    } catch (error) {
      check.status = 'error';
      check.error = error.message;
    }

    return check;
  }

  checkDOMState() {
    const check = {
      name: 'DOM Elements',
      status: 'checking',
      details: {}
    };

    const elements = {
      loginContainer: document.querySelector('.max-w-md'),
      devContainer: document.querySelector('.dev-accounts-floating-container'),
      devDropdown: document.querySelector('.dev-accounts-dropdown'),
      toggleButton: document.querySelector('button[title*="development accounts"]'),
      closeButton: document.querySelector('.close-button'),
      adminCard: document.querySelector('.admin-avatar'),
      superCard: document.querySelector('.super-avatar')
    };

    check.details = Object.entries(elements).reduce((acc, [key, element]) => {
      acc[key] = {
        exists: !!element,
        visible: element ? window.getComputedStyle(element).display !== 'none' : false
      };
      return acc;
    }, {});

    check.status = 'success';
    return check;
  }

  checkCSSState() {
    const check = {
      name: 'CSS Styles',
      status: 'checking',
      details: {}
    };

    const container = document.querySelector('.dev-accounts-floating-container');
    const dropdown = document.querySelector('.dev-accounts-dropdown');

    if (container) {
      const containerStyles = window.getComputedStyle(container);
      check.details.container = {
        display: containerStyles.display,
        visibility: containerStyles.visibility,
        opacity: containerStyles.opacity,
        zIndex: containerStyles.zIndex,
        position: containerStyles.position,
        pointerEvents: containerStyles.pointerEvents
      };
    }

    if (dropdown) {
      const dropdownStyles = window.getComputedStyle(dropdown);
      check.details.dropdown = {
        display: dropdownStyles.display,
        visibility: dropdownStyles.visibility,
        opacity: dropdownStyles.opacity,
        height: dropdownStyles.height,
        transform: dropdownStyles.transform
      };
    }

    check.status = 'success';
    return check;
  }

  checkStorageState() {
    const check = {
      name: 'LocalStorage State',
      status: 'checking',
      details: {}
    };

    check.details = {
      modalState: localStorage.getItem('dev-accounts-modal-state'),
      allKeys: Object.keys(localStorage).filter(key => key.includes('modal') || key.includes('dev'))
    };

    check.status = 'success';
    return check;
  }

  checkEventHandlers() {
    const check = {
      name: 'Event Handlers',
      status: 'checking',
      details: {}
    };

    const toggleButton = document.querySelector('button[title*="development accounts"]');
    const closeButton = document.querySelector('.close-button');

    check.details = {
      toggleButtonHasListeners: toggleButton ? toggleButton.onclick !== null : false,
      closeButtonHasListeners: closeButton ? closeButton.onclick !== null : false,
      keyboardShortcut: 'Ctrl+Shift+D should trigger modal'
    };

    check.status = 'success';
    return check;
  }

  checkExtensionInterference() {
    const check = {
      name: 'Extension Interference',
      status: 'checking',
      details: {}
    };

    // Check for common extension interference patterns
    const extensionElements = document.querySelectorAll('[class*="extension"], [id*="extension"], [class*="chrome-"], [id*="chrome-"]');

    // Check for extension blocker interference
    const blockerInterference = this.detectBlockerInterference();

    check.details = {
      extensionElementsFound: extensionElements.length,
      possibleInterference: extensionElements.length > 0,
      extensionBlockerActive: typeof window.extensionBlocker !== 'undefined',
      blockerInterference: blockerInterference,
      modalBeingBlocked: blockerInterference.modalBlocked
    };

    check.status = 'success';
    return check;
  }

  // Detect if extension blocker is interfering with modal
  detectBlockerInterference() {
    const interference = {
      blockerActive: typeof window.extensionBlocker !== 'undefined',
      modalBlocked: false,
      blockerStats: null,
      recommendations: []
    };

    if (interference.blockerActive) {
      interference.blockerStats = window.extensionBlocker.getStats();

      // Check if modal elements are missing but should exist
      const container = document.querySelector('.dev-accounts-floating-container');
      const dropdown = document.querySelector('.dev-accounts-dropdown');
      const savedState = localStorage.getItem('dev-accounts-modal-state');

      if (savedState === 'true' && container && !dropdown) {
        interference.modalBlocked = true;
        interference.recommendations.push('Extension blocker may be removing modal elements');
        interference.recommendations.push('Check extension blocker whitelist settings');
      }

      // Check blocker's blocked count - if very high, might be over-aggressive
      if (interference.blockerStats.blockedCount > 10) {
        interference.recommendations.push('Extension blocker might be over-aggressive');
        interference.recommendations.push('Consider adjusting blocker sensitivity');
      }

    return interference;
  }

  printDiagnosis(diagnosis) {
    if (import.meta.env.DEV) {
      console.log('\n📋 Modal Diagnosis Report');
    if (import.meta.env.DEV) {
      console.log('========================');
    if (import.meta.env.DEV) {
      console.log(`Time: ${diagnosis.timestamp}`);
    if (import.meta.env.DEV) {
      console.log(`Environment: ${diagnosis.environment}`);
    if (import.meta.env.DEV) {
      console.log('');
    }

    diagnosis.checks.forEach((check, index) => {
      const icon = check.status === 'success' ? '' : check.status === 'error' ? '❌' : '⏳';
      if (import.meta.env.DEV) {
        console.log(`${index + 1}. ${icon} ${check.name}`);
      }

      if (check.error) {
        if (import.meta.env.DEV) {
          console.log(`   Error: ${check.error}`);
        }

      if (check.details && Object.keys(check.details).length > 0) {
        if (import.meta.env.DEV) {
          console.log('   Details:', check.details);
      if (import.meta.env.DEV) {
        console.log('');
      }
    });

  // Auto-fix common modal issues
  autoFix() {
    if (import.meta.env.DEV) {
      console.log('Auto-fixing modal issues...');
    }

    const fixes = [];

    // Fix 0: Check and fix extension blocker interference
    const blockerInterference = this.detectBlockerInterference();
    if (blockerInterference.modalBlocked) {
      fixes.push('Detected extension blocker interference - applying fix');
      this.fixBlockerInterference();
    }

    // Fix 1: Ensure DOM elements exist
    if (!document.querySelector('.dev-accounts-floating-container')) {
      fixes.push('DOM elements missing - forcing Vue re-render');
      this.forceVueRerender();
    }

    // Fix 2: Reset localStorage state
    const currentState = localStorage.getItem('dev-accounts-modal-state');
    if (currentState !== 'true') {
      fixes.push('Setting localStorage state to true');
      localStorage.setItem('dev-accounts-modal-state', 'true');
    }

    // Fix 3: Force show modal
    if (window.showDevModal) {
      fixes.push('Calling window.showDevModal()');
      window.showDevModal();
    }

    // Fix 4: Add CSS overrides if needed
    if (!this.hasProperCSS()) {
      fixes.push('Adding CSS overrides');
      this.addCSSOverrides();
    }

    // Fix 5: Trigger keyboard shortcut
    fixes.push('Simulating Ctrl+Shift+D shortcut');
    this.simulateKeyboardShortcut();

    if (import.meta.env.DEV) {
      console.log('Applied fixes:', fixes);
    }

    // Verify fixes after delay
    setTimeout(() => {
      this.verifyFixes();
    }, 1000);
  }

  // Fix extension blocker interference
  fixBlockerInterference() {
    if (import.meta.env.DEV) {
      console.log('🛡️ Fixing extension blocker interference...');
    }

    // Temporarily disable DOM monitoring if possible
    if (window.extensionBlocker && typeof window.extensionBlocker.pauseMonitoring === 'function') {
      window.extensionBlocker.pauseMonitoring();
      if (import.meta.env.DEV) {
        console.log('Paused extension blocker monitoring');
      }

    // Add specific whitelist protection
    this.addBlockerWhitelist();

    // Force re-render with protection
    setTimeout(() => {
      if (window.showDevModal) {
        window.showDevModal();
      }
    }, 200);
  }

  // Add specific whitelist protection for modal elements
  addBlockerWhitelist() {
    // Create a custom data attribute to mark protected elements
    const protectedSelector = '.dev-accounts-floating-container, .dev-accounts-dropdown, .account-card, .fill-button';
    const protectedElements = document.querySelectorAll(protectedSelector);

    protectedElements.forEach(element => {
      element.setAttribute('data-fechatter-protected', 'true');
      element.setAttribute('data-modal-element', 'true');
    });

    if (import.meta.env.DEV) {
      console.log(`Protected ${protectedElements.length} modal elements from blocker`);
    }

  forceVueRerender() {
    if (window.debugModal) {
      window.debugModal();
    }

    // Force trigger reactive updates
    const event = new Event('resize');
    window.dispatchEvent(event);
  }

  hasProperCSS() {
    const container = document.querySelector('.dev-accounts-floating-container');
    if (!container) return false;

    const styles = window.getComputedStyle(container);
    return styles.zIndex === '9999';
  }

  addCSSOverrides() {
    const style = document.createElement('style');
    style.id = 'modal-debug-fix-styles';
    style.textContent = `
      .dev-accounts-floating-container {
        position: absolute !important;
        z-index: 9999 !important;
        display: block !important;
      }
      .dev-accounts-dropdown {
        display: block !important;
        visibility: visible !important;
        opacity: 1 !important;
      }
    `;
    document.head.appendChild(style);
  }

  simulateKeyboardShortcut() {
    const event = new KeyboardEvent('keydown', {
      key: 'D',
      ctrlKey: true,
      shiftKey: true,
      bubbles: true
    });
    document.dispatchEvent(event);
  }

  verifyFixes() {
    if (import.meta.env.DEV) {
      console.log('Verifying fixes...');
    }

    const container = document.querySelector('.dev-accounts-floating-container');
    const dropdown = document.querySelector('.dev-accounts-dropdown');

    const verification = {
      containerExists: !!container,
      dropdownExists: !!dropdown,
      containerVisible: container ? window.getComputedStyle(container).display !== 'none' : false,
      dropdownVisible: dropdown ? window.getComputedStyle(dropdown).display !== 'none' : false,
      localStorageCorrect: localStorage.getItem('dev-accounts-modal-state') === 'true'
    };

    if (import.meta.env.DEV) {
      console.log('Verification results:', verification);
    }

    const allFixed = Object.values(verification).every(v => v === true);
    if (import.meta.env.DEV) {
      console.log(allFixed ? 'All fixes successful!' : 'WARNING: Some issues remain');
    }

    return verification;
  }

  // Quick access methods
  show() {
    if (window.showDevModal) {
      window.showDevModal();
      setTimeout(() => this.verifyFixes(), 500);
    } else {
      if (import.meta.env.DEV) {
        console.log('ERROR: window.showDevModal not available');
      }

  hide() {
    if (window.hideDevModal) {
      window.hideDevModal();
    }

  reset() {
    if (window.resetDevModal) {
      window.resetDevModal();
    }

// Create global instance
const modalDebugFix = new ModalDebugFix();

// Auto-initialize in development
if (import.meta.env.DEV) {
  modalDebugFix.init();

  // Make available globally
  window.modalDebugFix = modalDebugFix;

  // Quick access shortcuts
  window.fixModal = () => modalDebugFix.autoFix();
  window.diagnoseModal = () => modalDebugFix.diagnose();

  if (import.meta.env.DEV) {
    console.log('Modal Debug Fix loaded');
  if (import.meta.env.DEV) {
    console.log('   - window.modalDebugFix - Full API');
  if (import.meta.env.DEV) {
    console.log('   - window.fixModal() - Auto-fix issues');
  if (import.meta.env.DEV) {
    console.log('   - window.diagnoseModal() - Run diagnosis');
  }

export default modalDebugFix; 