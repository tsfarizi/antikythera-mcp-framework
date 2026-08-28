#!/usr/bin/env node
// post-transpile-rename — rename the generic jco-emitted core modules to
// semantic names and rewrite the corresponding URL literals inside
// npm/antikythera-sdk/component/antikythera-sdk.js.
//
// jco transpile emits opaque numbered modules (the historical pattern this
// script migrates away from):
//   antikythera-sdk.core.wasm, antikythera-sdk.core2.wasm .. core6.wasm,
//   referenced only as literal './antikythera-sdk.coreN.wasm' inside the
//   generated loader. Those numbers carry no domain meaning and shift when
//   composition changes, so consumers cannot tell which module implements
//   which interface.
//
// Official semantic contract (fixed names):
//   antikythera-sdk.runner.core.wasm                 — main SDK agent-runner
//   antikythera-sdk.tool-registry.core.wasm          — toolrunner/tool-registry impl
//   antikythera-sdk.logic-hooks-passthrough.core.wasm— default-hooks/logic-hooks impl
//   antikythera-sdk.wasi-clocks.core.wasm            — WASI clocks (wall/monotonic) shim
//   antikythera-sdk.wasi-streams.core.wasm           — WASI streams (check-write/write) shim
//   antikythera-sdk.wasi-filesystem.core.wasm        — WASI filesystem (blocking-flush) shim
//
// Classification is EMPIRICAL (content-derived, never name-derived): each
// *.core*.wasm is scored by counting ASCII byte-signature occurrences, then
// slots are filled in fixed precedence:
//   1. runner                  : largest file containing 'antikythera:agent-sdk/runner'
//   2. tool-registry           : highest combined count of 'tool-registry' +
//                                'antikythera_toolrunner' among the rest
//   3. logic-hooks-passthrough : highest combined count of 'logic-hooks' +
//                                'runtime-hooks' + 'antikythera_default_hooks'
//   4. support-N               : everything else, numbered by original order
//
// Byte evidence recorded when this classification was first derived (probe
// over the composed bundle, counts = substring occurrences in raw bytes):
//   core.wasm  : 'antikythera_sdk' x123, 'antikythera:agent-sdk/runner' x50,
//                'tool-registry' x4, 'logic-hooks' x3            -> runner
//   core2.wasm : 'wasi:' x22 (no role signature at all)           -> support
//   core3.wasm : 'tool-registry' x11, 'antikythera_toolrunner' x25 -> tool-registry
//   core4.wasm : 'wasi:' x20 (no role signature at all)           -> support
//   core5.wasm : 'logic-hooks' x8, 'antikythera_default_hooks' x2  -> logic-hooks-passthrough
//   core6.wasm : 'wasi:' x19 (no role signature at all)           -> support
//
// Idempotent: files are classified purely by content; a file whose current
// name already equals its content-derived target is skipped, so a second run
// reports "no changes". Ambiguous classification (two equally strong
// candidates for one slot) fails closed with a score matrix plus the longest
// distinctive ASCII strings per file so a human can adjudicate.
//
// Usage: node scripts/post-transpile-rename.mjs [component-dir]
//   (component-dir defaults to npm/antikythera-sdk/component)
//
// Node >= 18, ESM, zero dependencies.

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const defaultComponentDir = path.join(repoRoot, 'npm', 'antikythera-sdk', 'component');
const loaderBasename = 'antikythera-sdk.js';

const NAME_RUNNER = 'antikythera-sdk.runner.core.wasm';
const NAME_TOOL_REGISTRY = 'antikythera-sdk.tool-registry.core.wasm';
const NAME_LOGIC_HOOKS = 'antikythera-sdk.logic-hooks-passthrough.core.wasm';
const NAME_WASI_CLOCKS = 'antikythera-sdk.wasi-clocks.core.wasm';
const NAME_WASI_STREAMS = 'antikythera-sdk.wasi-streams.core.wasm';
const NAME_WASI_FILESYSTEM = 'antikythera-sdk.wasi-filesystem.core.wasm';

// Old-name literal emitted by jco into the loader; rewritten via renameMap.
const LOADER_LITERAL = /\.\/antikythera-sdk\.core\d*\.wasm/g;

const WASM_FILE = /^antikythera-sdk\..*core\d*\.wasm$/;
const CORE_INDEX = /core(\d*)\.wasm$/;

const SIGNATURES = [
  ['runner-iface', 'antikythera:agent-sdk/runner'],
  ['tool-registry', 'tool-registry'],
  ['logic-hooks', 'logic-hooks'],
  ['runtime-hooks', 'runtime-hooks'],
  ['wasi-imports', 'wasi:'],
  ['component-type', 'component-type'],
  ['crate-toolrunner', 'antikythera_toolrunner'],
  ['crate-default-hooks', 'antikythera_default_hooks'],
  ['crate-sdk', 'antikythera_sdk'],
];

const SIG = Object.fromEntries(SIGNATURES.map(([key, text]) => [key, text]));

function countOccurrences(buffer, asciiText) {
  const needle = Buffer.from(asciiText, 'ascii');
  let count = 0;
  let index = buffer.indexOf(needle);
  while (index !== -1) {
    count += 1;
    index = buffer.indexOf(needle, index + 1);
  }
  return count;
}

function scoreSignatures(buffer) {
  const scores = new Map();
  for (const [key, text] of SIGNATURES) {
    scores.set(key, countOccurrences(buffer, text));
  }
  return scores;
}

// Distinct printable ASCII runs, longest first — diagnostic aid for humans
// adjudicating an ambiguous classification.
function longestAsciiStrings(buffer, limit) {
  const runs = new Set();
  let start = -1;
  for (let i = 0; i <= buffer.length; i += 1) {
    const printable = i < buffer.length && buffer[i] >= 0x20 && buffer[i] <= 0x7e;
    if (printable && start === -1) {
      start = i;
    } else if (!printable && start !== -1) {
      if (i - start >= 8) runs.add(buffer.toString('ascii', start, i));
      start = -1;
    }
  }
  return [...runs]
    .sort((a, b) => b.length - a.length || (a < b ? -1 : 1))
    .slice(0, limit);
}

function readInventory(componentDir) {
  return fs.readdirSync(componentDir)
    .filter((name) => WASM_FILE.test(name))
    .sort()
    .map((name) => {
      const buffer = fs.readFileSync(path.join(componentDir, name));
      return {
        currentName: name,
        size: buffer.length,
        buffer,
        scores: scoreSignatures(buffer),
      };
    });
}

function describeScores(scores) {
  return SIGNATURES
    .map(([key]) => [key, scores.get(key)])
    .filter(([, count]) => count > 0)
    .map(([key, count]) => `'${SIG[key]}' x${count}`)
    .join(', ') || '(no signatures)';
}

function pickStrongest(candidates, strengthOf, strengthLabel) {
  let best = null;
  let bestStrength = -1;
  let tie = false;
  for (const record of candidates) {
    const strength = strengthOf(record);
    if (strength > bestStrength) {
      best = record;
      bestStrength = strength;
      tie = false;
    } else if (strength === bestStrength) {
      tie = true;
    }
  }
  return { winner: best, strength: bestStrength, tie, strengthLabel };
}

function originalOrderKey(record) {
  const match = CORE_INDEX.exec(record.currentName);
  const digits = match ? match[1] : '';
  // Numbered jco names keep their composition order; already-semantic names
  // sort after them, lexicographically, so numbering stays deterministic.
  return digits !== ''
    ? [0, Number.parseInt(digits, 10)]
    : [1, Number.NaN, record.currentName];
}

function classify(inventory) {
  const assignments = new Map(); // record -> { name, basis }
  const pool = [...inventory];

  // Slot 1 — runner: the largest module carrying the agent-sdk/runner iface.
  const runnerCandidates = pool.filter((r) => r.scores.get('runner-iface') > 0);
  if (runnerCandidates.length > 0) {
    const pick = pickStrongest(runnerCandidates, (r) => r.size, 'bytes');
    if (pick.tie) {
      return { ambiguity: { slot: 'runner', rule: 'largest file matching \'antikythera:agent-sdk/runner\'', candidates: runnerCandidates } };
    }
    assignments.set(pick.winner, {
      name: NAME_RUNNER,
      basis: `runner slot: '${SIG['runner-iface']}' x${pick.winner.scores.get('runner-iface')}, largest match (${pick.winner.size} bytes)`,
    });
    pool.splice(pool.indexOf(pick.winner), 1);
  }

  // Slot 2 — tool-registry: strongest toolrunner signal among the rest.
  const registryCandidates = pool.filter((r) =>
    r.scores.get('tool-registry') > 0 || r.scores.get('crate-toolrunner') > 0);
  if (registryCandidates.length > 0) {
    const pick = pickStrongest(
      registryCandidates,
      (r) => r.scores.get('tool-registry') + r.scores.get('crate-toolrunner'),
      'signature hits',
    );
    if (pick.tie) {
      return { ambiguity: { slot: 'tool-registry', rule: 'highest combined \'tool-registry\' + \'antikythera_toolrunner\' count', candidates: registryCandidates } };
    }
    assignments.set(pick.winner, {
      name: NAME_TOOL_REGISTRY,
      basis: `tool-registry slot: '${SIG['tool-registry']}' x${pick.winner.scores.get('tool-registry')} + '${SIG['crate-toolrunner']}' x${pick.winner.scores.get('crate-toolrunner')}`,
    });
    pool.splice(pool.indexOf(pick.winner), 1);
  }

  // Slot 3 — logic-hooks passthrough: strongest hooks signal among the rest.
  const hooksCandidates = pool.filter((r) =>
    r.scores.get('logic-hooks') > 0 || r.scores.get('runtime-hooks') > 0 || r.scores.get('crate-default-hooks') > 0);
  if (hooksCandidates.length > 0) {
    const pick = pickStrongest(
      hooksCandidates,
      (r) => r.scores.get('logic-hooks') + r.scores.get('runtime-hooks') + r.scores.get('crate-default-hooks'),
      'signature hits',
    );
    if (pick.tie) {
      return { ambiguity: { slot: 'logic-hooks-passthrough', rule: 'highest combined \'logic-hooks\' + \'runtime-hooks\' + \'antikythera_default_hooks\' count', candidates: hooksCandidates } };
    }
    assignments.set(pick.winner, {
      name: NAME_LOGIC_HOOKS,
      basis: `logic-hooks slot: '${SIG['logic-hooks']}' x${pick.winner.scores.get('logic-hooks')} + '${SIG['crate-default-hooks']}' x${pick.winner.scores.get('crate-default-hooks')}`,
    });
    pool.splice(pool.indexOf(pick.winner), 1);
  }

  // Slot 4 — wasi shims: unclassified internal WASI adapters, descriptive names.
  // Descriptive mapping (short, stable, content-derived):
  //   wasi-clocks      — unique holder of wall-clock/monotonic-clock
  //   wasi-streams     — streams write/check-write without blocking-flush
  //   wasi-filesystem  — filesystem + blocking-flush holder
  const wasiClocks = pool.filter((r) => r.scores.get('wasi-imports') > 0 && (r.buffer.toString('ascii').includes('wall-clock') || r.buffer.toString('ascii').includes('monotonic-clock')));
  const wasiFilesystem = pool.filter((r) => r.buffer.toString('ascii').includes('blocking-flush'));
  const wasiStreams = pool.filter((r) => !wasiClocks.includes(r) && !wasiFilesystem.includes(r));
  const ordered = [...wasiClocks, ...wasiStreams, ...wasiFilesystem];
  // Fallback to original order if classification ambiguous (e.g. wit-bindgen grouping changed)
  const supportPool = ordered.length === pool.length ? ordered : pool.sort((a, b) => {
    const ka = originalOrderKey(a);
    const kb = originalOrderKey(b);
    return ka[0] - kb[0] || (ka[0] === 0 ? ka[1] - kb[1] : String(ka[2]).localeCompare(String(kb[2])));
  });
  const nameMap = [NAME_WASI_CLOCKS, NAME_WASI_STREAMS, NAME_WASI_FILESYSTEM];
  supportPool.forEach((record, index) => {
    const name = nameMap[index] || `antikythera-sdk.wasi-shim-${index + 1}.core.wasm`;
    assignments.set(record, {
      name,
      basis: `wasi shim -> ${name.split('.')[1]}: ${describeScores(record.scores)}`,
    });
  });

  return { assignments };
}

function reportAmbiguity(ambiguity, inventory) {
  console.error(`AMBIGUOUS classification for slot '${ambiguity.slot}'`);
  console.error(`rule: ${ambiguity.rule}`);
  console.error('');
  console.error('score matrix (occurrence counts):');
  const header = 'file'.padEnd(46) + SIGNATURES.map(([key]) => key.slice(0, 10).padStart(11)).join('');
  console.error(header);
  for (const record of inventory) {
    const row = record.currentName.padEnd(46)
      + SIGNATURES.map(([key]) => String(record.scores.get(key)).padStart(11)).join('');
    console.error(row);
  }
  console.error('');
  for (const record of inventory) {
    console.error(`${record.currentName} (${record.size} bytes) — top distinct ASCII strings:`);
    for (const text of longestAsciiStrings(record.buffer, 20)) {
      console.error(`  ${text}`);
    }
    console.error('');
  }
}

function planRenames(inventory, assignments, componentDir) {
  const targets = new Map(); // newName -> currentName
  const renames = [];
  for (const record of inventory) {
    const target = assignments.get(record).name;
    if (targets.has(target)) {
      return { error: `rename collision: '${targets.get(target)}' and '${record.currentName}' both map to '${target}'` };
    }
    targets.set(target, record.currentName);
    if (target !== record.currentName) {
      const targetPath = path.join(componentDir, target);
      if (fs.existsSync(targetPath)) {
        return { error: `rename collision: target '${target}' already exists on disk but is not part of the classified set` };
      }
      renames.push({ from: record.currentName, to: target });
    }
  }
  return { renames };
}

function rewriteLoader(componentDir, renameMap) {
  const loaderPath = path.join(componentDir, loaderBasename);
  if (!fs.existsSync(loaderPath)) {
    console.warn(`WARNING: '${loaderBasename}' not found in ${componentDir};`);
    console.warn('WARNING: URL literals were NOT rewritten (bundle not transpiled yet?).');
    return { rewritten: false, occurrences: 0 };
  }
  const source = fs.readFileSync(loaderPath, 'utf8');
  const unknownLiterals = [];
  let occurrences = 0;
  const rewritten = source.replace(LOADER_LITERAL, (literal) => {
    occurrences += 1;
    const mapped = renameMap.get(literal.slice(2));
    if (!mapped) {
      unknownLiterals.push(literal);
      return literal;
    }
    return `./${mapped}`;
  });
  if (unknownLiterals.length > 0) {
    return { error: `loader references modules absent from the classified set: ${[...new Set(unknownLiterals)].join(', ')}` };
  }
  if (rewritten !== source) {
    fs.writeFileSync(loaderPath, rewritten);
  }
  return { rewritten: true, occurrences };
}

function main() {
  const componentDir = process.argv[2] ? path.resolve(process.argv[2]) : defaultComponentDir;

  if (!fs.existsSync(componentDir) || !fs.readdirSync(componentDir).some((name) => WASM_FILE.test(name))) {
    console.log(`post-transpile-rename: no '*.core*.wasm' modules in ${componentDir}`);
    console.log('nothing to do — component bundle not built yet (run `task transpile` first).');
    process.exitCode = 0;
    return;
  }

  const inventory = readInventory(componentDir);
  const result = classify(inventory);
  if (result.ambiguity) {
    reportAmbiguity(result.ambiguity, inventory);
    process.exitCode = 1;
    return;
  }

  const plan = planRenames(inventory, result.assignments, componentDir);
  if (plan.error) {
    console.error(`post-transpile-rename: refusing partial state — ${plan.error}`);
    process.exitCode = 1;
    return;
  }

  console.log(`post-transpile-rename: scanned ${inventory.length} core module(s) in ${componentDir}`);
  console.log('');
  console.log('OLD NAME -> NEW NAME (basis)');
  for (const record of inventory) {
    const assignment = result.assignments.get(record);
    const moved = assignment.name === record.currentName ? '' : '  [RENAMED]';
    console.log(`  ${record.currentName}`);
    console.log(`    -> ${assignment.name}${moved}`);
    console.log(`       ${assignment.basis}`);
  }

  for (const { from, to } of plan.renames) {
    fs.renameSync(path.join(componentDir, from), path.join(componentDir, to));
  }

  const renameMap = new Map([...result.assignments].map(([record, assignment]) => [record.currentName, assignment.name]));
  const loader = rewriteLoader(componentDir, renameMap);
  if (loader.error) {
    console.error(`post-transpile-rename: ${loader.error}`);
    process.exitCode = 1;
    return;
  }

  console.log('');
  console.log(`renamed: ${plan.renames.length}, unchanged (already semantic): ${inventory.length - plan.renames.length}`);
  if (loader.rewritten) {
    console.log(`loader literals rewritten: ${loader.occurrences} occurrence(s) in ${loaderBasename}`);
  }
  console.log(plan.renames.length === 0 && (!loader.rewritten || loader.occurrences === 0)
    ? 'result: OK — no changes'
    : 'result: OK');
  process.exitCode = 0;
}

main();
