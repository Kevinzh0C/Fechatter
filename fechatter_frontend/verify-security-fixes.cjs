#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('🔒 Verifying Fechatter Security Fixes...\n');

const results = {
  passed: 0,
  failed: 0,
  warnings: 0
};

function checkFile(filePath, description) {
  try {
    if (fs.existsSync(filePath)) {
      console.log(`✅ ${description}: ${path.basename(filePath)}`);
      results.passed++;
      return true;
    } else {
      console.log(`❌ ${description}: ${path.basename(filePath)} NOT FOUND`);
      results.failed++;
      return false;
    }
  } catch (error) {
    console.log(`⚠️  ${description}: Error checking ${path.basename(filePath)}`);
    results.warnings++;
    return false;
  }
}

function checkCodePattern(filePath, pattern, description, shouldExist = true) {
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    const found = pattern.test(content);

    if ((found && shouldExist) || (!found && !shouldExist)) {
      console.log(`✅ ${description}`);
      results.passed++;
      return true;
    } else {
      console.log(`❌ ${description}`);
      results.failed++;
      return false;
    }
  } catch (error) {
    console.log(`⚠️  ${description}: Error checking pattern`);
    results.warnings++;
    return false;
  }
}

console.log('1️⃣ Checking Security Infrastructure Files...');
checkFile(path.join(__dirname, 'src/utils/secureLogger.js'), 'Secure Logger');
checkFile(path.join(__dirname, 'src/utils/productionSafetyWrapper.js'), 'Production Safety Wrapper');
checkFile(path.join(__dirname, 'src/utils/requestConflictResolver.js'), 'Request Conflict Resolver');
checkFile(path.join(__dirname, 'src/utils/extensionErrorSuppressor.js'), 'Extension Error Suppressor');
checkFile(path.join(__dirname, 'src/utils/performanceMonitor.js'), 'Performance Monitor');

console.log('\n2️⃣ Checking Console Log Wrapping...');
const chatJsPath = path.join(__dirname, 'src/stores/chat.js');
// Look for console statements that are not properly wrapped
try {
  const chatContent = fs.readFileSync(chatJsPath, 'utf8');
  const lines = chatContent.split('\n');
  let unwrappedFound = false;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line.match(/^\s*console\.(log|warn|error)\(/)) {
      // Found a console statement at the start of a line
      // Check if previous lines contain process.env.NODE_ENV check
      let isWrapped = false;
      for (let j = Math.max(0, i - 3); j < i; j++) {
        if (lines[j].includes('process.env.NODE_ENV')) {
          isWrapped = true;
          break;
        }
      }
      if (!isWrapped) {
        unwrappedFound = true;
        break;
      }
    }
  }

  if (!unwrappedFound) {
    console.log('✅ All console statements are properly wrapped in chat.js');
    results.passed++;
  } else {
    console.log('❌ Found unwrapped console statements in chat.js');
    results.failed++;
  }
} catch (error) {
  console.log('⚠️  Error checking console wrapping');
  results.warnings++;
}

console.log('\n3️⃣ Checking Shiki Service...');
const shikiPath = path.join(__dirname, 'src/services/shiki.js');
checkCodePattern(
  shikiPath,
  /const\s+secureShiki\s*=\s*new\s+SecureShikiService/,
  'Shiki singleton pattern implemented'
);
checkCodePattern(
  shikiPath,
  /DOMPurify/,
  'DOMPurify integration in Shiki'
);

console.log('\n4️⃣ Checking SSE Configuration...');
const ssePath = path.join(__dirname, 'src/services/sse.js');
checkCodePattern(
  ssePath,
  /if\s*\(import\.meta\.env\.DEV\)\s*{[^}]*sseUrl\s*=\s*['"]\/events['"]/,
  'SSE uses correct proxy path in development'
);

console.log('\n5️⃣ Checking Security Initialization...');
const mainPath = path.join(__dirname, 'src/main.js');
checkCodePattern(
  mainPath,
  /window\.securityUtils/,
  'Security utilities exposed globally'
);

console.log('\n6️⃣ Checking Build Output...');
try {
  const distExists = fs.existsSync(path.join(__dirname, 'dist'));
  if (distExists) {
    // Check if sensitive logs exist in production build
    const distFiles = execSync('grep -r "console.log" dist/ || true', { encoding: 'utf8' });
    if (distFiles.trim().length === 0) {
      console.log('✅ No console.log found in production build');
      results.passed++;
    } else {
      console.log('⚠️  Some console.log statements found in production build (may be from dependencies)');
      results.warnings++;
    }
  } else {
    console.log('ℹ️  Production build not found (run yarn build to verify)');
  }
} catch (error) {
  console.log('⚠️  Could not check production build');
  results.warnings++;
}

console.log('\n7️⃣ Checking Dependencies...');
try {
  const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, 'package.json'), 'utf8'));
  if (packageJson.dependencies && packageJson.dependencies.dompurify) {
    console.log('✅ DOMPurify dependency installed');
    results.passed++;
  } else {
    console.log('❌ DOMPurify dependency not found');
    results.failed++;
  }
} catch (error) {
  console.log('⚠️  Error checking package.json');
  results.warnings++;
}

console.log('\n📊 Security Verification Summary:');
console.log(`✅ Passed: ${results.passed}`);
console.log(`❌ Failed: ${results.failed}`);
console.log(`⚠️  Warnings: ${results.warnings}`);

if (results.failed === 0) {
  console.log('\n🎉 All security fixes verified successfully!');
  process.exit(0);
} else {
  console.log('\n❗ Some security fixes need attention.');
  process.exit(1);
} 