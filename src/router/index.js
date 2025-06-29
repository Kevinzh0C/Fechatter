const redirectTarget = isTokenValid ? '/home' : '/login';

if (import.meta.env.DEV) {
  console.log('🔍 [ROUTER] Root redirect decision:', {
    hasLocalToken: !!token,
    hasSessionToken: !!sessionToken,
    hasExpiry: !!finalExpiry,
    rememberMe,
    isValid: isTokenValid,
    currentTime: new Date().getTime(),
    expiryTime: finalExpiry ? parseInt(finalExpiry) : null,
    redirectTo: redirectTarget
  });
}

return redirectTarget; 