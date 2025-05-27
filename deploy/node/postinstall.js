#!/usr/bin/env node

import { promises as fs } from 'fs';
import path from 'path';
import https from 'https';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import zlib from 'zlib';
import { execSync } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const VERSION = '0.1.0';
const GITHUB_REPO = 'alexjbuck/verset';

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

// Windows binaries end with .exe
const binaryName = process.platform === 'win32' ? 'verset.exe' : 'verset';

// Determine package name for this platform
const platformSpecificPackageName = BINARY_DISTRIBUTION_PACKAGES[`${process.platform}-${getArch()}`];

// Compute the path we want to emit the fallback binary to
const fallbackBinaryPath = path.join(__dirname, 'lib', binaryName);

function makeRequest(url) {
  return new Promise((resolve, reject) => {
    https.get(url, (response) => {
      if (response.statusCode >= 200 && response.statusCode < 300) {
        const chunks = [];
        response.on('data', (chunk) => chunks.push(chunk));
        response.on('end', () => {
          resolve(Buffer.concat(chunks));
        });
      } else if (
        response.statusCode >= 300 &&
        response.statusCode < 400 &&
        response.headers.location
      ) {
        // Follow redirects
        makeRequest(response.headers.location).then(resolve, reject);
      } else {
        reject(
          new Error(
            `npm responded with status code ${response.statusCode} when downloading the package!`
          )
        );
      }
    }).on('error', (error) => {
      reject(error);
    });
  });
}

function extractFileFromTarball(tarballBuffer, filepath) {
  // Tar archives are organized in 512 byte blocks.
  // Blocks can either be header blocks or data blocks.
  // Header blocks contain file names of the archive in the first 100 bytes, terminated by a null byte.
  // The size of a file is contained in bytes 124-135 of a header block and in octal format.
  // The following blocks will be data blocks containing the file.
  let offset = 0;
  while (offset < tarballBuffer.length) {
    const header = tarballBuffer.subarray(offset, offset + 512);
    offset += 512;

    const fileName = header.toString('utf-8', 0, 100).replace(/\0.*/g, '');
    const fileSize = parseInt(header.toString('utf-8', 124, 136).replace(/\0.*/g, ''), 8);

    if (fileName === filepath) {
      return tarballBuffer.subarray(offset, offset + fileSize);
    }

    // Clamp offset to the upper multiple of 512
    offset = (offset + fileSize + 511) & ~511;
  }
}

async function downloadBinaryFromNpm() {
  // Download the tarball of the right binary distribution package
  // Note: Scoped packages need URL encoding
  const encodedPackageName = platformSpecificPackageName.replace('/', '%2f');
  const tarballDownloadBuffer = await makeRequest(
    `https://registry.npmjs.org/${encodedPackageName}/-/${platformSpecificPackageName.replace('@verset/', 'verset-')}-${VERSION}.tgz`
  );

  const tarballBuffer = zlib.gunzipSync(tarballDownloadBuffer);

  // Extract binary from package and write to disk
  await fs.mkdir(path.dirname(fallbackBinaryPath), { recursive: true });
  
  await fs.writeFile(
    fallbackBinaryPath,
    extractFileFromTarball(tarballBuffer, `package/bin/${binaryName}`),
    { mode: 0o755 } // Make binary file executable
  );
}

async function downloadBinaryFromGitHub() {
  // Fallback to downloading from GitHub releases
  const assetName = getAssetName();
  const url = `https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${assetName}`;
  
  console.log(`Downloading verset from GitHub: ${url}...`);
  const data = await makeRequest(url);
  
  await fs.mkdir(path.dirname(fallbackBinaryPath), { recursive: true });
  
  if (assetName.endsWith('.tar.gz')) {
    // Extract tar.gz
    const tempFile = path.join(__dirname, 'temp.tar.gz');
    await fs.writeFile(tempFile, data);
    
    // Use tar command
    execSync(`tar -xzf "${tempFile}" -C "${path.dirname(fallbackBinaryPath)}" "${binaryName}"`);
    await fs.unlink(tempFile);
  } else if (assetName.endsWith('.zip')) {
    // For Windows, use PowerShell to extract
    const tempFile = path.join(__dirname, 'temp.zip');
    await fs.writeFile(tempFile, data);
    
    execSync(`powershell -command "Expand-Archive -Path '${tempFile}' -DestinationPath '${path.dirname(fallbackBinaryPath)}' -Force"`);
    await fs.unlink(tempFile);
  }
  
  // Make binary executable on Unix
  if (process.platform !== 'win32') {
    await fs.chmod(fallbackBinaryPath, 0o755);
  }
}

function getAssetName() {
  const platform = process.platform;
  const arch = getArch();

  if (platform === 'darwin') {
    if (arch === 'arm64') {
      return 'verset-aarch64-apple-darwin.tar.gz';
    } else {
      return 'verset-x86_64-apple-darwin.tar.gz';
    }
  } else if (platform === 'linux') {
    if (arch === 'arm64') {
      return 'verset-aarch64-unknown-linux-gnu.tar.gz';
    } else {
      return 'verset-x86_64-unknown-linux-gnu.tar.gz';
    }
  } else if (platform === 'win32') {
    return 'verset-x86_64-pc-windows-msvc.zip';
  } else {
    throw new Error(`Unsupported platform: ${platform} ${arch}`);
  }
}

function isPlatformSpecificPackageInstalled() {
  try {
    // Resolving will fail if the optionalDependency was not installed
    require.resolve(`${platformSpecificPackageName}/bin/${binaryName}`);
    return true;
  } catch (e) {
    return false;
  }
}

async function install() {
  if (!platformSpecificPackageName) {
    console.error('Platform not supported!');
    console.error(`Platform: ${process.platform}, Architecture: ${process.arch}`);
    process.exit(0); // Don't fail npm install
  }

  // Skip downloading the binary if it was already installed via optionalDependencies
  if (isPlatformSpecificPackageInstalled()) {
    console.log('Platform specific package already installed via optionalDependencies.');
    return;
  }

  console.log('Platform specific package not found. Downloading binary as fallback...');
  
  try {
    // Try downloading from npm first
    await downloadBinaryFromNpm();
    console.log('verset binary downloaded successfully from npm!');
  } catch (npmError) {
    console.log('Failed to download from npm, trying GitHub releases...');
    try {
      await downloadBinaryFromGitHub();
      console.log('verset binary downloaded successfully from GitHub!');
    } catch (githubError) {
      console.error('Failed to download verset binary from both npm and GitHub:');
      console.error('npm error:', npmError.message);
      console.error('GitHub error:', githubError.message);
      console.error('');
      console.error('You can manually download verset from:');
      console.error(`https://github.com/${GITHUB_REPO}/releases`);
      console.error('');
      console.error('Note: optionalDependencies may be disabled. Consider enabling them for better reliability.');
      // Don't fail npm install
    }
  }
}

// Only run install if this is the main module
if (process.argv[1] === __filename) {
  install().catch((error) => {
    console.error('Unexpected error during installation:', error);
    // Don't fail npm install
    process.exit(0);
  });
} 