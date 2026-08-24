#!/usr/bin/env node
// pack-smoke — publish gate for the npm and python packages.
//
// Resolves every file the packages PROMISE to ship (exports map including
// browser/default/node conditions, files array, main/types) against the
// working tree, then builds a python wheel and asserts the PEP 561 marker,
// the inline stub, the WASM artifact, and the jco component bundle are all
// inside it. Exit 0 only when everything resolves; any gap exits 1 with an
// explicit message so `prepublishOnly` fails closed.
//
// Node >= 18, ESM, zero dependencies.

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const pkgDir = path.join(repoRoot, 'npm', 'antikythera-sdk');
const pythonCmd = process.platform === 'win32' ? 'python' : 'python3';

const failures = [];

function fail(message) {
  failures.push(message);
}

function checkTarget(relSpec, source) {
  const rel = relSpec.replace(/^\.\//, '');
  if (rel.includes('..')) {
    fail(`${source}: target escapes package dir: ${relSpec}`);
    return;
  }
  const abs = path.join(pkgDir, rel);
  let stat;
  try {
    stat = fs.statSync(abs);
  } catch {
    fail(`${source}: missing on disk: ${rel}`);
    return;
  }
  if (stat.isDirectory()) {
    let entries;
    try {
      entries = fs.readdirSync(abs);
    } catch (e) {
      fail(`${source}: cannot read directory ${rel}: ${e.message}`);
      return;
    }
    if (entries.length === 0) {
      fail(`${source}: directory is empty: ${rel}`);
    }
  }
}

function collectExportTargets(node, out) {
  if (typeof node === 'string') {
    out.add(node);
    return;
  }
  if (node && typeof node === 'object') {
    // Recurses through every condition key: browser/default/node and future ones.
    for (const value of Object.values(node)) collectExportTargets(value, out);
  }
}

// --- npm --------------------------------------------------------------------

const manifest = JSON.parse(fs.readFileSync(path.join(pkgDir, 'package.json'), 'utf8'));

if (manifest.main) checkTarget(manifest.main, 'main');
if (manifest.types) checkTarget(manifest.types, 'types');
for (const entry of manifest.files ?? []) checkTarget(entry, `files[${entry}]`);

const exportTargets = new Set();
collectExportTargets(manifest.exports, exportTargets);
for (const target of exportTargets) checkTarget(target, `exports[${target}]`);

// shell:true with a single literal command string: npm is a .cmd shim on
// Windows that cannot be spawned directly; the string carries no interpolated
// input, so the DEP0190 concatenation concern does not apply.
const pack = spawnSync('npm pack --dry-run --json', {
  cwd: pkgDir,
  encoding: 'utf8',
  shell: true,
});
if (pack.status !== 0 || !pack.stdout.trim()) {
  fail(`npm pack --dry-run --json failed:\n${pack.stderr || pack.stdout}`);
} else {
  let packedFiles;
  try {
    packedFiles = JSON.parse(pack.stdout)[0].files.map((f) => f.path);
  } catch (e) {
    fail(`cannot parse npm pack --json output: ${e.message}`);
    packedFiles = [];
  }
  console.log(`npm tarball would carry ${packedFiles.length} entries`);
}

// --- python -----------------------------------------------------------------

const pyTmp = fs.mkdtempSync(path.join(os.tmpdir(), 'pack-smoke-py-'));
const wheelBuild = spawnSync(
  pythonCmd,
  ['-m', 'pip', 'wheel', '--no-deps', '-w', pyTmp, path.join(repoRoot, 'python')],
  {
    cwd: repoRoot,
    encoding: 'utf8',
  },
);
if (wheelBuild.status !== 0) {
  console.error('pack-smoke: python wheel build failed (python tooling required):');
  console.error(wheelBuild.stderr || wheelBuild.stdout);
  process.exit(1);
}

const wheels = fs.readdirSync(pyTmp).filter((f) => f.endsWith('.whl'));
if (wheels.length !== 1) {
  fail(`expected exactly one wheel in tmp dir, got: ${wheels.join(', ') || '(none)'}`);
} else {
  const listZip = spawnSync(
    pythonCmd,
    [
      '-c',
      'import sys, zipfile; print("\\n".join(zipfile.ZipFile(sys.argv[1]).namelist()))',
      path.join(pyTmp, wheels[0]),
    ],
    { encoding: 'utf8' },
  );
  if (listZip.status !== 0) {
    console.error('pack-smoke: could not inspect wheel contents (python tooling required):');
    console.error(listZip.stderr || listZip.stdout);
    process.exit(1);
  }
  const names = new Set(listZip.stdout.split('\n').map((n) => n.trim()).filter(Boolean));
  for (const required of [
    'antikythera_agent/py.typed',
    'antikythera_agent/__init__.pyi',
    'antikythera_agent/antikythera.wasm',
  ]) {
    if (!names.has(required)) fail(`wheel ${wheels[0]}: missing ${required}`);
  }
  const hasComponent = [...names].some((n) => n.startsWith('antikythera_agent/component/'));
  if (!hasComponent) fail(`wheel ${wheels[0]}: missing antikythera_agent/component/ bundle`);
  console.log(`python wheel ${wheels[0]} carries ${names.size} entries`);
}

// --- verdict ----------------------------------------------------------------

fs.rmSync(pyTmp, { recursive: true, force: true });

if (failures.length > 0) {
  console.error('pack-smoke FAILED:');
  for (const message of failures) console.error(`  - ${message}`);
  process.exit(1);
}
console.log('pack-smoke OK: every packaged target resolves');
