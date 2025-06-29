/**
 * 统一文件URL处理系统
 */

import { useAuthStore } from '@/stores/auth';

function getWorkspaceId(fileInput, options) {
  const authStore = useAuthStore();
  return options.workspaceId || authStore.user?.workspace_id || fileInput?.workspace_id || 2;
}

function isHashPath(str) {
  const parts = str.split('/');
  return parts.length >= 3 && parts[0].length === 3 && parts[1].length === 3 && parts[2].includes('.');
}

function isSimpleFilename(str) {
  return !str.includes('/') && str.includes('.');
}

function constructHashUrl(filename, workspaceId) {
  // 🚨 CRITICAL FIX: Empty filename check to prevent /files/2/ incomplete URLs
  if (!filename || filename.trim() === '') {
    console.error('ERROR: [FileUrlHandler] Empty filename provided, cannot construct URL');
    return null;
  }

  if (isHashPath(filename)) {
    return '/api/files/' + workspaceId + '/' + filename;
  }
  const cleanFilename = filename.replace(/^.*\//, '');

  // 🚨 CRITICAL FIX: Validate clean filename
  if (!cleanFilename || cleanFilename.trim() === '') {
    console.error('ERROR: [FileUrlHandler] Invalid filename after cleaning:', filename);
    return null;
  }

  if (cleanFilename.length >= 10) {
    const hash1 = cleanFilename.substring(0, 3);
    const hash2 = cleanFilename.substring(3, 6);
    // 🚨 CRITICAL FIX: Remove hash prefix to match actual storage format
    const finalFilename = cleanFilename.substring(6); // Remove hash1+hash2 prefix
    return '/api/files/' + workspaceId + '/' + hash1 + '/' + hash2 + '/' + finalFilename;
  }
  return '/api/files/' + workspaceId + '/' + cleanFilename;
}

function normalizeUrlString(url, workspaceId) {
  // 🚨 CRITICAL FIX: Handle empty/null URLs
  if (!url || url.trim() === '') {
    console.error('ERROR: [FileUrlHandler] Empty URL provided');
    return null;
  }

  console.log('[FileUrlHandler] Normalizing URL:', url, 'workspace:', workspaceId);

  // CRITICAL FIX: Handle ANY /download/ format FIRST - extract filename and construct proper hash URL
  if (url.includes('/download/')) {
    const filename = url.split('/download/')[1];
    console.log('[FileUrlHandler] Fixing download URL:', url, '-> filename:', filename);
    if (filename && filename.length >= 10) {
      const hash1 = filename.substring(0, 3);
      const hash2 = filename.substring(3, 6);
      // 🚨 CRITICAL FIX: Remove hash prefix from filename to match actual storage
      const cleanFilename = filename.substring(6); // Remove first 6 chars (hash1+hash2)
      const fixedUrl = '/api/files/' + workspaceId + '/' + hash1 + '/' + hash2 + '/' + cleanFilename;
      console.log('[FileUrlHandler] Fixed URL (removed hash prefix):', fixedUrl);
      return fixedUrl;
    }
    return constructHashUrl(filename, workspaceId);
  }

  const workspacePattern = '/' + workspaceId + '/';
  const isApiFiles = url.startsWith('/api/files/');
  const isFiles = url.startsWith('/files/');
  const hasWorkspace = url.includes(workspacePattern);

  // CORRECTED: /api/files/ format is already correct
  if (isApiFiles && hasWorkspace) {
    console.log('[FileUrlHandler] Already correct /api/files/ format:', url);
    return url;
  }

  // CORRECTED: Convert /files/ to /api/files/
  if (isFiles && hasWorkspace) {
    const converted = url.replace('/files/', '/api/files/');
    console.log('[FileUrlHandler] Converted /files/ to /api/files/:', url, '→', converted);
    return converted;
  }

  if (isApiFiles) {
    const pathPart = url.substring(11);
    const result = '/api/files/' + workspaceId + '/' + pathPart;
    console.log('[FileUrlHandler] Added workspace:', url, '→', result);
    return result;
  }

  if (isFiles) {
    const parts = url.split('/');
    if (parts.length >= 3) {
      const filename = parts.slice(2).join('/');
      return constructHashUrl(filename, workspaceId);
    }
  }
  if (isHashPath(url)) {
    return '/api/files/' + workspaceId + '/' + url;
  }
  if (url.includes('/app/data/')) {
    const cleanPath = url.replace(/^.*\/app\/data\//, '');
    return '/api/files/' + workspaceId + '/' + cleanPath;
  }
  if (isSimpleFilename(url)) {
    return constructHashUrl(url, workspaceId);
  }
  if (url.startsWith('http') || url.startsWith('blob:')) {
    return url;
  }
  console.warn('WARNING: [FileUrlHandler] Unknown URL format:', url);
  return constructHashUrl(url, workspaceId);
}

function normalizeFileObject(file, workspaceId) {
  console.log('[FileUrlHandler] Processing file object:', file);

  const candidates = [
    file.file_url,
    file.url,
    file.path,
    file.filename,
    file.file_name,
    file.name
  ].filter(Boolean);

  console.log('[FileUrlHandler] URL candidates:', candidates);

  for (const candidate of candidates) {
    console.log('[FileUrlHandler] Testing candidate:', candidate);
    const result = normalizeUrlString(candidate, workspaceId);
    if (result) {
      console.log('[FileUrlHandler] Successfully normalized:', candidate, '→', result);
      return result;
    }
  }
  console.error('ERROR: [FileUrlHandler] No valid URL found in file object:', file);
  return null;
}

export function getStandardFileUrl(fileInput, options = {}) {
  try {
    const workspaceId = getWorkspaceId(fileInput, options);
    let result;

    if (typeof fileInput === 'string') {
      result = normalizeUrlString(fileInput, workspaceId);
    } else if (typeof fileInput === 'object' && fileInput !== null) {
      result = normalizeFileObject(fileInput, workspaceId);
    } else {
      console.error('ERROR: [FileUrlHandler] Invalid file input:', fileInput);
      return null;
    }

    // 🚨 CRITICAL VALIDATION: Ensure no /download/ URLs escape
    if (result && result.includes('/download/')) {
      console.error('🚨 CRITICAL: /download/ URL detected in output, forcing fix:', result);

      // Extract filename and force fix
      const downloadMatch = result.match(/\/download\/(.+)$/);
      if (downloadMatch) {
        const filename = downloadMatch[1];
        if (filename.length >= 10) {
          const hash1 = filename.substring(0, 3);
          const hash2 = filename.substring(3, 6);
          const cleanFilename = filename.substring(6);
          result = '/api/files/' + workspaceId + '/' + hash1 + '/' + hash2 + '/' + cleanFilename;
          console.log('EMERGENCY FIX applied:', result);
        }
      }
    }

    // 🚨 VALIDATION: Ensure filename doesn't contain hash prefix
    if (result && result.includes('/api/files/')) {
      const parts = result.split('/');
      if (parts.length >= 6) {
        const filename = parts[5];
        const hash1 = parts[3];
        const hash2 = parts[4];

        if (filename.startsWith(hash1 + hash2)) {
          const cleanFilename = filename.substring(6);
          result = '/api/files/' + workspaceId + '/' + hash1 + '/' + hash2 + '/' + cleanFilename;
          console.log('HASH PREFIX FIX applied:', result);
        }
      }
    }

    console.log('[FileUrlHandler] Final output:', result);
    return result;
  } catch (error) {
    console.error('ERROR: [FileUrlHandler] Error processing file URL:', error);
    return null;
  }
}

export function debugFileUrlHandler(fileInput, options = {}) {
  console.group('[FileUrlHandler] URL Debug');
  console.log('Input:', fileInput);
  const result = getStandardFileUrl(fileInput, options);
  console.log('Output:', result);
  console.groupEnd();
  return result;
}

export class FileUrlHandler {
  getStandardUrl(fileInput, options = {}) {
    return getStandardFileUrl(fileInput, options);
  }

  debugUrlConversion(fileInput, options = {}) {
    return debugFileUrlHandler(fileInput, options);
  }
}

export const fileUrlHandler = new FileUrlHandler();
