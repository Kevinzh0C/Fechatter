# Auth Loop Fix - Complete DAG Chain

## Root Cause Analysis

```mermaid
graph TD
    A[User on /login page] --> B[SSE service active]
    B --> C[SSE sends presence update]
    C --> D[API request with token]
    D --> E[Server returns 401]
    E --> F[API interceptor catches 401]
    F --> G[handleAuthFailure called]
    G --> H[authStore.logout called]
    H --> I[Attempts to navigate to /login]
    I --> J[Already on /login - Navigation fails]
    J --> K[Error: Avoided redundant navigation]
    K --> L[Error handler triggered]
    L --> F
    
    style E fill:#f99,stroke:#333,stroke-width:2px
    style J fill:#f99,stroke:#333,stroke-width:2px
    style K fill:#f99,stroke:#333,stroke-width:2px
```

## Solution Implementation

```mermaid
graph TD
    A[Fix 1: API Error Handler] --> B[Check current route]
    B --> C[Skip logout if on auth page]
    
    D[Fix 2: Auth Store] --> E[Check route before navigation]
    E --> F[Prevent redundant navigation]
    
    G[Fix 3: SSE Manager] --> H[Monitor route changes]
    H --> I[Disconnect SSE on auth pages]
    
    J[Fix 4: SSE Service] --> K[Check route before requests]
    K --> L[Skip presence updates on auth pages]
    
    M[Integration] --> N[Initialize SSE Manager in main.js]
    N --> O[Automatic connection lifecycle]
    
    style A fill:#9f9,stroke:#333,stroke-width:2px
    style D fill:#9f9,stroke:#333,stroke-width:2px
    style G fill:#9f9,stroke:#333,stroke-width:2px
    style J fill:#9f9,stroke:#333,stroke-width:2px
```

## Request Flow After Fix

```mermaid
sequenceDiagram
    participant U as User
    participant R as Router
    participant API as API Service
    participant Auth as Auth Store
    participant SSE as SSE Service
    participant SM as SSE Manager
    
    U->>R: Navigate to /login
    R->>SM: Route change detected
    SM->>SSE: Disconnect SSE
    SSE-->>SM: Connection closed
    
    Note over SSE: No presence updates sent
    
    alt If 401 Error Occurs
        API->>API: Check current route
        API->>API: Already on /login
        API->>API: Clear tokens only
        API-->>U: No navigation attempted
    end
    
    U->>Auth: Login successful
    Auth->>SM: Auth state changed
    SM->>SSE: Connect SSE
    SSE->>API: Send presence update
    API-->>SSE: Success
```

## Files Modified

### 1. **api.js** - Enhanced handleAuthFailure
```javascript
// Check if already on login/register page to prevent redirect loops
const currentPath = window.location.pathname;
if (currentPath === '/login' || currentPath === '/register') {
  console.log('🔐 [AUTH] Already on auth page, skipping logout redirect');
  // Clear tokens but don't redirect
  tokenManager.clearTokens();
  return Promise.reject(error);
}
```

### 2. **auth.js** - Smart logout navigation
```javascript
// Redirect to login only if not already there
if (window.$router) {
  const currentRoute = window.$router.currentRoute.value;
  if (currentRoute.path !== '/login' && currentRoute.path !== '/register') {
    window.$router.push('/login');
  }
}
```

### 3. **sseConnectionManager.js** - New connection lifecycle manager
- Monitors route changes
- Automatically disconnects SSE on auth pages
- Reconnects SSE when leaving auth pages

### 4. **sse.js** - Route-aware presence updates
```javascript
// Check if we're on auth pages - don't send presence updates
if (window.location.pathname === '/login' || 
    window.location.pathname === '/register' ||
    window.location.pathname === '/forgot-password' ||
    window.location.pathname === '/reset-password') {
  console.log('📡 [SSE] Skipping presence update on auth page');
  return;
}
```

### 5. **main.js** - Integration point
```javascript
// Initialize SSE connection manager
const { useAuthStore } = await import('@/stores/auth');
const authStore = useAuthStore();
sseConnectionManager.initialize(router, authStore);
```

## Benefits

1. **No More Loops**: 401 errors on auth pages don't trigger navigation loops
2. **Clean Separation**: SSE connections properly managed based on auth state
3. **Performance**: No unnecessary API calls on auth pages
4. **User Experience**: Smooth transitions between auth and app states

## Testing

```javascript
// Run on login page:
window.testAuthLoopFix()

// Check SSE status:
window.sseConnectionManager.isOnAuthRoute()
window.realtimeCommunicationService.getConnectionState()

// Test navigation:
window.testNavigationBehavior()
```

## Occam's Razor Applied

- Simple route checks vs complex state machines
- Direct path comparison vs elaborate routing logic
- Clear separation of concerns vs mixed responsibilities
- Minimal code changes for maximum impact 