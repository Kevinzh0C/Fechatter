#!/usr/bin/env node

/**
 * Copy Configuration Files Script
 * Copies YAML configuration files to public directory for web access
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const projectRoot = path.resolve(__dirname, '..');
const configDir = path.join(projectRoot, 'config');
const publicConfigDir = path.join(projectRoot, 'public', 'config');

// Ensure public/config directory exists
if (!fs.existsSync(publicConfigDir)) {
  fs.mkdirSync(publicConfigDir, { recursive: true });
  console.log('📁 Created public/config directory');
}

// Copy configuration files
const configFiles = [
  'development.yml',
  'production.yml'
];

console.log('🔄 Copying configuration files...');

configFiles.forEach(fileName => {
  const sourcePath = path.join(configDir, fileName);
  const destPath = path.join(publicConfigDir, fileName);

  try {
    if (fs.existsSync(sourcePath)) {
      fs.copyFileSync(sourcePath, destPath);
      console.log(`Copied ${fileName} to public/config/`);
    } else {
      console.warn(`WARNING: Source file not found: ${fileName}`);
    }
  } catch (error) {
    console.error(`ERROR: Failed to copy ${fileName}:`, error.message);
  }
});

console.log('🎉 Configuration files copied successfully!');

// Verify files are accessible
console.log('\n📋 Available configurations:');
configFiles.forEach(fileName => {
  const destPath = path.join(publicConfigDir, fileName);
  if (fs.existsSync(destPath)) {
    const stats = fs.statSync(destPath);
    console.log(`  ✓ ${fileName} (${stats.size} bytes)`);
  }
}); 