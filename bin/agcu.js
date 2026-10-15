#!/usr/bin/env node

const { execFileSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

function getBinaryName() {
  const platform = os.platform();
  const arch = os.arch();

  const map = {
    'darwin-arm64': 'agcu-macos-arm64',
    'darwin-x64': 'agcu-macos-x64',
    'linux-x64': 'agcu-linux-x64',
    'linux-arm64': 'agcu-linux-arm64',
    'win32-x64': 'agcu-windows-x64.exe',
  };

  const key = `${platform}-${arch}`;
  return map[key] || null;
}

function findBinary() {
  const binDir = __dirname;

  const platformBinary = getBinaryName();
  if (platformBinary) {
    const platformPath = path.join(binDir, platformBinary);
    if (fs.existsSync(platformPath)) return platformPath;
  }

  const executable = os.platform() === 'win32' ? 'agcu.exe' : 'agcu';
  const devPaths = [
    path.join(binDir, '..', 'cli', 'target', 'release', executable),
    path.join(binDir, '..', 'cli', 'target', 'debug', executable),
  ];

  for (const p of devPaths) {
    if (fs.existsSync(p)) return p;
  }

  try {
    const candidate = execFileSync('which', ['agcu'], { encoding: 'utf8' }).trim();
    // An npm bin shim may resolve to this launcher; don't recursively launch it.
    if (fs.realpathSync(candidate) !== fs.realpathSync(__filename)) return candidate;
  } catch {}

  console.error(
    'agcu binary not found.\n\n' +
      'Either:\n' +
      '  1. Build from source: cd cli && cargo build --release\n' +
      '  2. Install via cargo: cargo install --git https://github.com/elithecho/agcu agcu\n',
  );
  process.exit(1);
}

const binary = findBinary();
const args = process.argv.slice(2);

// Self-heal: actions/upload-artifact@v4 strips the exec bit on POSIX, so the
// tarball we publish can land with mode 0644 and fail silently. Re-apply 755
// before every invocation — cheap and idempotent. Ignore EPERM on read-only
// filesystems and on Windows where chmod is a no-op.
try {
  fs.chmodSync(binary, 0o755);
} catch {}

try {
  execFileSync(binary, args, {
    stdio: 'inherit',
    env: process.env,
  });
} catch (err) {
  // err.status is a number when the child ran and exited non-zero — the
  // binary already wrote its own stderr, so just propagate the code.
  // Otherwise spawn itself failed (EACCES, ENOENT, ...) and nothing reached
  // the user's terminal — surface the reason instead of silent-exiting.
  if (typeof err.status !== 'number') {
    console.error(`agcu: failed to run ${binary}`);
    console.error(`  ${err.code || 'error'}: ${err.message}`);
    console.error(`  try reinstalling: npm i -g agcu@latest`);
  }
  process.exit(typeof err.status === 'number' ? err.status : 1);
}
