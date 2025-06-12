/**
 * Extension Conflict User Guide
 * Provides guidance for users experiencing browser extension conflicts
 */

export const extensionConflictGuide = {
  title: 'Browser Extension Conflict Resolution',

  symptoms: [
    'Login errors with message channel references',
    'Asynchronous response timeout errors',
    'Unexpected Promise rejection errors',
    'Login process interruptions',
    'Console errors from content scripts',
    'fetchError: Failed to fetch from extensions',
    'Network request failures from browser extensions'
  ],

  conflictTypes: {
    message_channel: {
      name: 'Message Channel Timeout',
      description: 'Extension message listeners causing async response timeouts',
      severity: 'Medium'
    },
    network_request: {
      name: 'Extension Network Request Conflict',
      description: 'Browser extensions making failed network requests to app endpoints',
      severity: 'Low'
    },
    async_listener: {
      name: 'Async Listener Conflict',
      description: 'Extension listeners not properly handling async responses',
      severity: 'Medium'
    },
    general: {
      name: 'General Extension Interference',
      description: 'Various extension conflicts affecting app functionality',
      severity: 'Variable'
    }
  },

  solutions: [
    {
      title: 'Quick Fix - Incognito Mode',
      description: 'Try logging in using incognito/private browsing mode',
      steps: [
        'Open new incognito window (Ctrl+Shift+N or Cmd+Shift+N)',
        'Navigate to the login page',
        'Attempt login again'
      ],
      difficulty: 'Easy',
      effectiveFor: ['message_channel', 'network_request', 'async_listener', 'general']
    },
    {
      title: 'Disable Extensions Temporarily',
      description: 'Temporarily disable browser extensions to identify conflicts',
      steps: [
        'Go to chrome://extensions/ (Chrome) or about:addons (Firefox)',
        'Disable all extensions temporarily',
        'Try logging in again',
        'Re-enable extensions one by one to identify the problematic one'
      ],
      difficulty: 'Medium',
      effectiveFor: ['message_channel', 'network_request', 'async_listener', 'general']
    },
    {
      title: 'Check Network-Active Extensions',
      description: 'Extensions that make network requests may interfere with app communications',
      problematicExtensions: [
        'Password managers (LastPass, 1Password, Bitwarden)',
        'Security extensions (uBlock Origin, AdBlock Plus)',
        'Privacy extensions (Ghostery, Privacy Badger)',
        'Web scraping extensions',
        'Social media extensions',
        'Shopping assistants and price trackers',
        'VPN and proxy extensions',
        'Developer tools extensions'
      ],
      difficulty: 'Medium',
      effectiveFor: ['network_request', 'general']
    },
    {
      title: 'Check Login-Related Extensions',
      description: 'Common extensions that may interfere with login processes',
      problematicExtensions: [
        'Password managers (auto-fill conflicts)',
        'Form auto-fill extensions',
        'Login security extensions',
        'Two-factor authentication extensions'
      ],
      difficulty: 'Medium',
      effectiveFor: ['message_channel', 'async_listener']
    },
    {
      title: 'Clear Browser Data',
      description: 'Clear cookies and site data',
      steps: [
        'Go to browser settings',
        'Find "Privacy and Security" or "Clear browsing data"',
        'Select "Cookies and other site data"',
        'Clear data for the last hour',
        'Try logging in again'
      ],
      difficulty: 'Easy',
      effectiveFor: ['message_channel', 'network_request', 'general']
    },
    {
      title: 'Advanced - Extension Permission Review',
      description: 'Review and restrict extension permissions',
      steps: [
        'Go to chrome://extensions/ (Chrome) or about:addons (Firefox)',
        'Click "Details" on each extension',
        'Review "Site access" permissions',
        'Change "On all sites" to "On click" where possible',
        'Remove unnecessary permissions'
      ],
      difficulty: 'Advanced',
      effectiveFor: ['network_request', 'general']
    }
  ],

  prevention: [
    'Keep extensions updated',
    'Use fewer extensions during login',
    'Configure extension permissions carefully',
    'Use browser profiles for different purposes',
    'Regularly review extension permissions',
    'Avoid extensions with broad site access',
    'Test critical workflows in incognito mode'
  ],

  technicalInfo: {
    messageChannelError: {
      errorType: 'Message Channel Timeout',
      cause: 'Browser extension message listeners returning async promises without proper handling',
      impact: 'May cause login delays or failures',
      autoHandling: 'Errors are automatically detected and suppressed'
    },
    networkRequestError: {
      errorType: 'Extension Network Request Failure',
      cause: 'Browser extensions attempting to make network requests to app endpoints or third-party services',
      impact: 'Creates console noise but does not affect core application functionality',
      autoHandling: 'Errors are automatically detected, classified, and suppressed'
    },
    general: {
      errorType: 'Extension Interference',
      cause: 'Various browser extension conflicts with application functionality',
      impact: 'Variable impact depending on conflict type',
      autoHandling: 'Comprehensive error detection and conflict resolution guidance'
    }
  }
};

/**
 * Show user-friendly conflict guide
 */
export function showExtensionConflictGuide() {
  console.group('🔧 Browser Extension Conflict Guide');

  console.log('📋 Common Symptoms:');
  extensionConflictGuide.symptoms.forEach(symptom => {
    console.log(`  • ${symptom}`);
  });

  console.log('\n🔍 Conflict Types:');
  Object.entries(extensionConflictGuide.conflictTypes).forEach(([key, type]) => {
    console.log(`\n• ${type.name} (${type.severity} severity)`);
    console.log(`  ${type.description}`);
  });

  console.log('\n🛠️ Solutions by Effectiveness:');
  extensionConflictGuide.solutions.forEach((solution, index) => {
    console.log(`\n${index + 1}. ${solution.title} (${solution.difficulty})`);
    console.log(`   ${solution.description}`);
    console.log(`   Effective for: ${solution.effectiveFor.join(', ')}`);

    if (solution.steps) {
      solution.steps.forEach(step => {
        console.log(`   • ${step}`);
      });
    }

    if (solution.problematicExtensions) {
      console.log('   Common problematic extensions:');
      solution.problematicExtensions.forEach(ext => {
        console.log(`   • ${ext}`);
      });
    }
  });

  console.log('\n🛡️ Prevention Tips:');
  extensionConflictGuide.prevention.forEach(tip => {
    console.log(`  • ${tip}`);
  });

  console.log('\n📊 Technical Details:');
  Object.entries(extensionConflictGuide.technicalInfo).forEach(([key, info]) => {
    console.log(`\n${info.errorType}:`);
    console.log(`  Cause: ${info.cause}`);
    console.log(`  Impact: ${info.impact}`);
    console.log(`  Auto Handling: ${info.autoHandling}`);
  });

  console.groupEnd();
}

/**
 * Show specific conflict type guide
 */
export function showConflictTypeGuide(conflictType) {
  const typeInfo = extensionConflictGuide.conflictTypes[conflictType];
  if (!typeInfo) {
    console.warn(`Unknown conflict type: ${conflictType}`);
    return;
  }

  console.group(`🔧 ${typeInfo.name} Resolution Guide`);

  console.log(`Description: ${typeInfo.description}`);
  console.log(`Severity: ${typeInfo.severity}`);

  const relevantSolutions = extensionConflictGuide.solutions.filter(
    solution => solution.effectiveFor.includes(conflictType)
  );

  console.log('\n🛠️ Recommended Solutions:');
  relevantSolutions.forEach((solution, index) => {
    console.log(`\n${index + 1}. ${solution.title}`);
    console.log(`   ${solution.description}`);
    if (solution.steps) {
      solution.steps.forEach(step => {
        console.log(`   • ${step}`);
      });
    }
  });

  console.groupEnd();
}

/**
 * Create user notification for extension conflicts
 */
export function createConflictNotification(conflictType = 'general') {
  if (typeof window === 'undefined' || !window.Notification) {
    return;
  }

  const typeInfo = extensionConflictGuide.conflictTypes[conflictType] || extensionConflictGuide.conflictTypes.general;

  const notification = {
    title: `Browser Extension Conflict Detected`,
    message: `${typeInfo.name}: ${typeInfo.description}`,
    action: 'Click here for troubleshooting guide',
    persistent: false,
    conflictType
  };

  return notification;
}

// Expose to window for easy access
if (typeof window !== 'undefined') {
  window.showExtensionConflictGuide = showExtensionConflictGuide;
  window.showConflictTypeGuide = showConflictTypeGuide;
  window.extensionConflictGuide = extensionConflictGuide;
}

export default extensionConflictGuide; 