#!/usr/bin/env node

/**
 * Script to package platform-specific binaries for npm publishing.
 * 
 * This script should be run after building the Rust binaries for all platforms.
 * It will copy the binaries into the appropriate package directories.
 * 
 * Expected binary locations (relative to project root):
 * - target/x86_64-apple-darwin/release/verset
 * - target/aarch64-apple-darwin/release/verset
 * - target/x86_64-unknown-linux-gnu/release/verset
 * - target/aarch64-unknown-linux-gnu/release/verset
 * - target/x86_64-pc-windows-msvc/release/verset.exe
 */

import { promises as fs } from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const BINARY_MAPPINGS = [
  {
    source: '../../../target/x86_64-apple-darwin/release/verset',
    package: 'darwin-x64',
    dest: 'bin/verset'
  },
  {
    source: '../../../target/aarch64-apple-darwin/release/verset',
    package: 'darwin-arm64',
    dest: 'bin/verset'
  },
  {
    source: '../../../target/x86_64-unknown-linux-gnu/release/verset',
    package: 'linux-x64',
    dest: 'bin/verset'
  },
  {
    source: '../../../target/aarch64-unknown-linux-gnu/release/verset',
    package: 'linux-arm64',
    dest: 'bin/verset'
  },
  {
    source: '../../../target/x86_64-pc-windows-msvc/release/verset.exe',
    package: 'win32-x64',
    dest: 'bin/verset.exe'
  }
];

async function packageBinaries() {
  console.log('Packaging platform-specific binaries for npm...\n');
  
  for (const mapping of BINARY_MAPPINGS) {
    const sourcePath = path.join(__dirname, '..', mapping.source);
    const packageDir = path.join(__dirname, '..', 'packages', mapping.package);
    const destPath = path.join(packageDir, mapping.dest);
    const destDir = path.dirname(destPath);
    
    console.log(`Processing @verset/${mapping.package}...`);
    
    try {
      // Check if source binary exists
      await fs.access(sourcePath);
      
      // Create bin directory
      await fs.mkdir(destDir, { recursive: true });
      
      // Copy binary
      await fs.copyFile(sourcePath, destPath);
      
      // Make executable (except Windows)
      if (!mapping.dest.endsWith('.exe')) {
        await fs.chmod(destPath, 0o755);
      }
      
      console.log(`  ✓ Copied binary to ${mapping.dest}`);
    } catch (error) {
      console.error(`  ✗ Failed: ${error.message}`);
      console.error(`    Make sure the binary exists at: ${sourcePath}`);
    }
  }
  
  console.log('\nDone! You can now publish the packages:');
  console.log('  cd deploy/node/packages/<package-name> && npm publish --access public');
  console.log('\nNote: Scoped packages require --access public on first publish');
}

packageBinaries().catch(console.error); 