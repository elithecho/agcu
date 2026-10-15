import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { spawnSync } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import test from 'node:test';
import vm from 'node:vm';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8'));
const launcher = readFileSync(resolve(root, Object.values(pkg.bin)[0]), 'utf8');

test('release build runner propagates a Docker failure instead of reporting success', () => {
  const fixture = mkdtempSync(join(tmpdir(), 'agcu-build-'));
  try {
    mkdirSync(join(fixture, 'tools'));
    mkdirSync(join(fixture, 'docker'));
    writeFileSync(join(fixture, 'tools/uname'), '#!/bin/sh\necho Linux\n', { mode: 0o755 });
    writeFileSync(join(fixture, 'tools/docker'), '#!/bin/sh\nexit 23\n', { mode: 0o755 });
    const result = spawnSync('bash', [resolve(root, 'scripts/build-all.sh')], {
      cwd: fixture,
      env: { ...process.env, PATH: `${join(fixture, 'tools')}:${process.env.PATH}` },
      encoding: 'utf8',
      timeout: 10000,
    });
    assert.equal(result.status, 23, result.stdout + result.stderr);
    assert.ok(!result.stdout.includes('✅ Linux + Windows builds'));
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});

function run({ platform = 'darwin', arch = 'arm64', files = [], exit = 0, pathBinary } = {}) {
  const calls = [];
  const errors = [];
  const exits = [];
  const binaryDir = '/fixture/bin';
  const nodeProcess = {
    argv: ['node', `${binaryDir}/agcu.js`, 'screenshot', '--grid', '--path', 'safe shot.png'],
    env: { FIXTURE: 'isolated' },
    exit(code) {
      exits.push(code);
      throw new Error('fixture exit');
    },
  };
  const modules = {
    child_process: {
      execFileSync(binary, args, options) {
        if (binary === 'which') {
          if (pathBinary) return pathBinary + '\n';
          throw new Error('not on PATH');
        }
        calls.push({ binary, args, options });
        if (exit) throw Object.assign(new Error('fixture child error'), { status: exit });
      },
    },
    path: { join: (...parts) => resolve(...parts) },
    fs: {
      existsSync: (path) => files.includes(path),
      realpathSync: (path) => path,
      chmodSync() {},
    },
    os: { platform: () => platform, arch: () => arch },
  };
  try {
    vm.runInNewContext(launcher, {
      require: (name) => modules[name],
      __dirname: binaryDir,
      __filename: `${binaryDir}/agcu.js`,
      process: nodeProcess,
      console: { error: (message) => errors.push(message) },
    });
  } catch (error) {
    assert.equal(error.message, 'fixture exit');
  }
  return JSON.parse(JSON.stringify({ calls, errors, exits }));
}

test('public npm package exposes only the agcu command', () => {
  assert.equal(pkg.name, 'agcu');
  assert.deepEqual(pkg.bin, { agcu: './bin/agcu.js' });
  assert.equal(pkg.repository, 'https://github.com/elithecho/agcu');
});

for (const [platform, arch, artifact] of [
  ['darwin', 'arm64', 'agcu-macos-arm64'],
  ['darwin', 'x64', 'agcu-macos-x64'],
  ['linux', 'x64', 'agcu-linux-x64'],
  ['linux', 'arm64', 'agcu-linux-arm64'],
  ['win32', 'x64', 'agcu-windows-x64.exe'],
]) {
  test(`launcher selects ${artifact} and forwards exact arguments/environment`, () => {
    const result = run({
      platform,
      arch,
      files: [`/fixture/bin/${artifact}`, '/fixture/cli/target/debug/agcu'],
    });
    assert.deepEqual(result, {
      calls: [
        {
          binary: `/fixture/bin/${artifact}`,
          args: ['screenshot', '--grid', '--path', 'safe shot.png'],
          options: { stdio: 'inherit', env: { FIXTURE: 'isolated' } },
        },
      ],
      errors: [],
      exits: [],
    });
  });
}

test('launcher uses Cargo release before debug, then debug when release is absent', () => {
  for (const files of [
    ['/fixture/cli/target/release/agcu', '/fixture/cli/target/debug/agcu'],
    ['/fixture/cli/target/debug/agcu'],
  ]) {
    assert.equal(run({ files }).calls[0].binary, files[0]);
  }
});

test('launcher preserves native nonzero exit status', () => {
  const result = run({ files: ['/fixture/bin/agcu-macos-arm64'], exit: 7 });
  assert.deepEqual(result.exits, [7]);
  assert.deepEqual(result.errors, []);
  assert.equal(result.calls[0].binary, '/fixture/bin/agcu-macos-arm64');
});

test('Windows Cargo fallback uses the executable suffix', () => {
  const result = run({
    platform: 'win32',
    arch: 'x64',
    files: ['/fixture/cli/target/debug/agcu.exe'],
  });
  assert.equal(result.calls[0]?.binary, '/fixture/cli/target/debug/agcu.exe');
});

test('PATH fallback runs a native agcu but never recursively runs the npm launcher', () => {
  const native = run({ pathBinary: '/cargo/bin/agcu' });
  assert.equal(native.calls[0].binary, '/cargo/bin/agcu');
  const self = run({ pathBinary: '/fixture/bin/agcu.js' });
  assert.deepEqual(self.calls, []);
  assert.deepEqual(self.exits, [1]);
  assert.equal(self.errors[0].split('\n')[0], 'agcu binary not found.');
});

test('missing binary fails with source-install guidance instead of reporting success', () => {
  const result = run();
  assert.deepEqual(result.exits, [1]);
  assert.deepEqual(result.calls, []);
  assert.deepEqual(result.errors, [
    'agcu binary not found.\n\nEither:\n  1. Build from source: cd cli && cargo build --release\n  2. Install via cargo: cargo install --git https://github.com/elithecho/agcu agcu\n',
  ]);
});

test('shipped skill and distribution paths agree with the public package', () => {
  const skill = readFileSync(resolve(root, 'skills/agcu/SKILL.md'), 'utf8');
  assert.match(skill, /^---\nname: agcu\n/);
  assert.match(skill, /homepage: https:\/\/github.com\/elithecho\/agcu/);
  assert.ok(!skill.includes('agent-cu'));
  const builds =
    readFileSync(resolve(root, 'scripts/build-all.sh'), 'utf8') +
    readFileSync(resolve(root, 'docker/docker-compose.yml'), 'utf8');
  const bundle = readFileSync(resolve(root, 'scripts/bundle-npm.sh'), 'utf8');
  for (const artifact of [
    'agcu-macos-arm64',
    'agcu-macos-x64',
    'agcu-linux-x64',
    'agcu-linux-arm64',
    'agcu-windows-x64.exe',
  ]) {
    assert.ok(builds.includes(artifact));
    assert.ok(bundle.includes(artifact));
  }
});

test('screenshot smoke runner executes all seven checks against the renamed source binary', () => {
  const calls = [];
  const files = new Map();
  const output = '/fixture/evidence';
  const binary = resolve(root, 'cli/target/debug/agcu');
  const png = (grid) => {
    const data = Buffer.alloc(32);
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]).copy(data);
    data.write('IHDR', 12);
    data.writeUInt32BE(1100, 16);
    data.writeUInt32BE(1050, 20);
    data[31] = grid ? 1 : 0;
    return data;
  };
  const source = readFileSync(resolve(root, 'scripts/test-screenshot-grid.mjs'), 'utf8')
    .replace(/^import .*;\n/gm, '')
    .replace(
      'import.meta.url',
      JSON.stringify(pathToFileURL(resolve(root, 'scripts/test-screenshot-grid.mjs')).href),
    );
  vm.runInNewContext(source, {
    assert,
    Buffer,
    dirname,
    join,
    resolve,
    fileURLToPath,
    process: { platform: 'darwin', argv: ['node', 'script', 'Harmless Fixture', output] },
    console: { log() {} },
    existsSync: (path) => path === binary || files.has(path),
    mkdirSync() {},
    rmSync: (path) => files.delete(path),
    readFileSync: (path) => files.get(path),
    writeFileSync: (path, data) => files.set(path, data),
    spawnSync(executable, args) {
      calls.push({ executable, args });
      const grid = args.includes('--grid');
      const path = args.includes('--path')
        ? args[args.indexOf('--path') + 1]
        : '/fixture/default.png';
      if (path.includes('missing-parent') || args.includes('__agcu_missing_fixture_app__')) {
        return {
          status: 1,
          stderr: JSON.stringify({
            error: true,
            type: path.includes('missing-parent') ? 'platform_error' : 'application_not_found',
          }),
        };
      }
      files.set(path, png(grid));
      return {
        status: 0,
        stdout: JSON.stringify({
          success: true,
          message: `screenshot saved to ${path}${grid ? ' with coordinate grid (100 screen points)' : ''}`,
          path,
        }),
      };
    },
  });
  assert.equal(calls.length, 5);
  assert.ok(calls.every((call) => call.executable === binary));
  const result = JSON.parse(files.get(`${output}/smoke-results.json`));
  assert.deepEqual(result.checks, [
    'plain response',
    'grid response',
    'PNG dimensions',
    'changed output',
    'invalid path',
    'missing app',
    'default path',
  ]);
  assert.equal(result.binary, binary);
  assert.deepEqual(result.dimensions, { width: 1100, height: 1050 });
});
