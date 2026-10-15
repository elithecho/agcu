import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// Run only against a harmless app: node scripts/test-screenshot-grid.mjs App /path/to/artifacts
assert.equal(process.platform, 'darwin', 'The grid smoke test needs macOS');
const [app, directory] = process.argv.slice(2);
assert.ok(app && directory, 'Usage: node scripts/test-screenshot-grid.mjs App /path/to/artifacts');
const output = resolve(directory);
mkdirSync(output, { recursive: true });
const binary = resolve(dirname(fileURLToPath(import.meta.url)), '../cli/target/debug/agcu');
assert.ok(existsSync(binary), 'Build the source CLI with cd cli && cargo build');

function run(args, expectedStatus = 0) {
  const result = spawnSync(binary, args, { encoding: 'utf8', timeout: 15000 });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, expectedStatus, result.stderr);
  return JSON.parse(expectedStatus === 0 ? result.stdout : result.stderr);
}

function png(path) {
  const data = readFileSync(path);
  assert.deepEqual(data.subarray(0, 8), Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]));
  assert.equal(data.subarray(12, 16).toString(), 'IHDR');
  return { width: data.readUInt32BE(16), height: data.readUInt32BE(20) };
}

const plainPath = join(output, 'smoke-plain.png');
const gridPath = join(output, 'smoke-grid.png');
const plain = run(['screenshot', '-a', app, '--path', plainPath]);
const grid = run(['screenshot', '-a', app, '--path', gridPath, '--grid']);
assert.deepEqual(plain, {
  success: true,
  message: `screenshot saved to ${plainPath}`,
  path: plainPath,
});
assert.deepEqual(grid, {
  success: true,
  message: `screenshot saved to ${gridPath} with coordinate grid (100 screen points)`,
  path: gridPath,
});
assert.deepEqual(png(gridPath), png(plainPath));
assert.notDeepEqual(readFileSync(gridPath), readFileSync(plainPath), 'Grid must alter the output');

const invalidPath = join(output, 'missing-parent', 'failed.png');
assert.equal(existsSync(dirname(invalidPath)), false, 'Invalid-path fixture must remain absent');
const invalid = run(['screenshot', '-a', app, '--path', invalidPath, '--grid'], 1);
assert.equal(invalid.error, true);
assert.equal(invalid.type, 'platform_error');
assert.equal(existsSync(invalidPath), false);

const missingPath = join(output, 'missing-app.png');
const missing = run(
  ['screenshot', '-a', '__agcu_missing_fixture_app__', '--path', missingPath, '--grid'],
  1,
);
assert.equal(missing.error, true);
assert.equal(missing.type, 'application_not_found');
assert.equal(existsSync(missingPath), false);

const automatic = run(['screenshot', '-a', app, '--grid']);
assert.equal(automatic.success, true);
assert.deepEqual(png(automatic.path), png(gridPath));
rmSync(automatic.path);
const result = {
  binary,
  app,
  platform: process.platform,
  checks: [
    'plain response',
    'grid response',
    'PNG dimensions',
    'changed output',
    'invalid path',
    'missing app',
    'default path',
  ],
  dimensions: png(gridPath),
  artifacts: [plainPath, gridPath],
  invalid,
  missing,
};
writeFileSync(join(output, 'smoke-results.json'), `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify(result, null, 2));
