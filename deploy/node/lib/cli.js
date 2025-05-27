#!/usr/bin/env node

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { existsSync } from 'fs';
import { createRequire } from 'module';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const require = createRequire(import.meta.url);

// Lookup table for all platforms and binary distribution packages
const BINARY_DISTRIBUTION_PACKAGES = {
  'darwin-x64': '@verset/darwin-x64',
  'darwin-arm64': '@verset/darwin-arm64',
  'linux-x64': '@verset/linux-x64',
  'linux-arm64': '@verset/linux-arm64',
  'win32-x64': '@verset/win32-x64',
};

// Map Node.js arch to our arch naming
function getArch() {
  const arch = process.arch;
  if (arch === 'x64') return 'x64';
  if (arch === 'arm64') return 'arm64';
  if (arch === 'arm') return 'arm64'; // Assume arm is arm64
  return arch;
}

function getBinaryPath() {
  // Windows binaries end with .exe
  const binaryName = process.platform === 'win32' ? 'verset.exe' : 'verset';
  
  // Determine package name for this platform
  const platformSpecificPackageName = BINARY_DISTRIBUTION_PACKAGES[`${process.platform}-${getArch()}`];
  
  if (platformSpecificPackageName) {
    try {
      // Try to resolve the platform-specific package first (optionalDependencies)
      return require.resolve(`${platformSpecificPackageName}/bin/${binaryName}`);
    } catch (e) {
      // Fall through to fallback
    }
  }
  
  // Fallback to the binary downloaded by postinstall script
  const fallbackPath = join(__dirname, '..', 'lib', binaryName);
  if (existsSync(fallbackPath)) {
    return fallbackPath;
  }
  
  // Last resort - check if it's in the current lib directory
  const libPath = join(__dirname, binaryName);
  if (existsSync(libPath)) {
    return libPath;
  }
  
  return null;
}

const binary = getBinaryPath();

if (!binary) {
  console.error('Error: verset binary not found!');
  console.error('This can happen if:');
  console.error('1. The postinstall script failed to download the binary');
  console.error('2. optionalDependencies are disabled and the fallback download failed');
  console.error('');
  console.error('Try reinstalling with optionalDependencies enabled:');
  console.error('  npm install --no-save --no-optional=false');
  console.error('');
  console.error('Or download manually from:');
  console.error('  https://github.com/alexjbuck/verset/releases');
  process.exit(1);
}

// Spawn the binary with all arguments
const child = spawn(binary, process.argv.slice(2), {
  stdio: 'inherit',
  shell: false
});

child.on('exit', (code) => {
  process.exit(code || 0);
});

child.on('error', (err) => {
  console.error('Failed to run verset:', err);
  process.exit(1);
}); 