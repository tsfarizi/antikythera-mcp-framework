// http-bundle-loader.mjs — TEST HARNESS (bukan kode produksi, bukan bagian
// dari runtime client). U42: memungkinkan `node --test` mengeksekusi bundle
// jco yang di-serve oleh server Python NYATA melalui `componentBase`.
//
// Mengapa perlu:
//   1. Loader ESM default Node menolak specifier http: (ERR_UNSUPPORTED_
//      ESM_URL_SCHEME) — runtime client memanggil import(componentBase/entry).
//   2. Bundle jco di Node membaca WASM via fs.readFile(new URL(...,
//      import.meta.url)) — jadi import.meta.url WAJIB berupa URL file: agar
//      path wasm ter-resolve di filesystem.
//
// Mekanisme: `installHttpBundleLoader` memasang module hooks in-process
// (module.registerHooks, thread yang sama — state test terlihat langsung):
//   - resolve(): specifier http di bawah base yang dikonfigurasi (atau
//     specifier relatif dari modul yang sudah dimaterialisasi) di-fetch dari
//     server NYATA over HTTP, ditulis byte-identik ke tempDir, dan dipetakan
//     ke URL file:; asset `.wasm` yang dirujuk source bundle di-pre-fetch
//     over HTTP juga.
//   - load(): file temp dibaca sebagai ESM ('module').
// Bundle yang dieksekusi adalah SALINAN byte-identik dari yang di-serve
// Python (di-fetch over HTTP, direkam di registry) — BUKAN bundled path
// lokal `../component/`. Tanpa `componentBase` runtime tidak pernah menyentuh
// loader ini (registry tetap kosong) — itulah pembeda yang diuji.
//
// Amplop: base URL harus berasal dari server yang sedang hidup; semua fetch
// memakai globalThis.fetch saat hook berjalan (probe fetch test tetap
// melihatnya). URL di luar base TIDAK di-intercept (delegasi ke resolver
// bawaan). `deregister()` mencabut hook; `rm(tempDir)` adalah urusan pemanggil.

'use strict';

import { registerHooks } from 'node:module';
import { pathToFileURL, fileURLToPath } from 'node:url';
import path from 'node:path';
import fs from 'node:fs';

/** True bila URL absolut berada di bawah `baseUrl` (origin + path prefix). */
function isUnderBase(candidateHref, baseHref) {
  const base = new URL(baseHref);
  const candidate = new URL(candidateHref);
  if (candidate.origin !== base.origin) return false;
  const basePath = base.pathname.endsWith('/') ? base.pathname : `${base.pathname}/`;
  return candidate.pathname.startsWith(basePath);
}

/**
 * Instal loader materialisasi bundle http -> temp dir.
 * @param {object} options
 * @param {string} options.baseUrl - base URL bundle yang di-serve server
 *   (contoh `http://127.0.0.1:PORT/antikythera/v1/component/`).
 * @param {string} options.tempDir - direktori temp tempat file dimaterialisasi.
 * @returns {{ registry: { fetches: Array<object> }, deregister: Function }}
 */
export function installHttpBundleLoader({ baseUrl, tempDir }) {
  if (typeof baseUrl !== 'string' || !baseUrl) {
    throw new Error('http-bundle-loader: baseUrl is required');
  }
  if (typeof tempDir !== 'string' || !tempDir) {
    throw new Error('http-bundle-loader: tempDir is required');
  }
  const base = new URL(baseUrl);
  const basePath = base.pathname.endsWith('/') ? base.pathname : `${base.pathname}/`;
  const fileCache = new Map(); // httpHref -> absolute file path (sudah dimaterialisasi)
  const fetches = []; // {url, status, contentType, bytes, kind}

  // --- pemetaan path ----------------------------------------------------

  function httpHrefToFilePath(httpHref) {
    const parsed = new URL(httpHref);
    if (parsed.origin !== base.origin) {
      throw new Error(`http-bundle-loader: fetch outside served origin: ${httpHref}`);
    }
    if (!parsed.pathname.startsWith(basePath)) {
      throw new Error(`http-bundle-loader: fetch outside served base ${basePath}: ${httpHref}`);
    }
    const rel = parsed.pathname.slice(basePath.length);
    if (!rel || rel.split('/').some((seg) => seg === '..' || seg === '' || seg === '.')) {
      throw new Error(`http-bundle-loader: unsafe relative path '${rel}' in ${httpHref}`);
    }
    const filePath = path.join(tempDir, ...rel.split('/'));
    const root = path.resolve(tempDir);
    if (!path.resolve(filePath).startsWith(root + path.sep)) {
      throw new Error(`http-bundle-loader: escaped temp dir: ${httpHref}`);
    }
    return filePath;
  }

  function tempFilePathToHttpHref(filePath) {
    const resolved = path.resolve(filePath);
    const root = path.resolve(tempDir);
    if (!resolved.startsWith(root + path.sep)) return null;
    const rel = path.relative(root, resolved).split(path.sep).join('/');
    return new URL(rel, base.href).href;
  }

  // --- materialisasi ----------------------------------------------------

  async function materialize(httpHref) {
    const cached = fileCache.get(httpHref);
    if (cached) return cached;

    const filePath = httpHrefToFilePath(httpHref);
    let response;
    try {
      response = await fetch(httpHref);
    } catch (err) {
      throw new Error(
        `http-bundle-loader: fetch ${httpHref} failed: ${err instanceof Error ? err.message : String(err)}`,
      );
    }
    const bytes = Buffer.from(await response.arrayBuffer());
    if (!response.ok) {
      throw new Error(`http-bundle-loader: fetch ${httpHref} failed (HTTP ${response.status})`);
    }
    await fs.promises.mkdir(path.dirname(filePath), { recursive: true });
    await fs.promises.writeFile(filePath, bytes);

    const kind = httpHref.endsWith('.wasm') ? 'wasm' : 'js';
    fetches.push({
      url: httpHref,
      status: response.status,
      contentType: response.headers.get('content-type'),
      bytes: bytes.length,
      kind,
    });
    fileCache.set(httpHref, filePath);

    // Bundle jco di Node membaca WASM via fs.readFile(import.meta.url-rel).
    // Pre-fetch asset `.wasm` yang dirujuk source agar path file: ter-resolve.
    if (kind === 'js') {
      const source = bytes.toString('utf8');
      const wasmRefs = [...new Set(source.match(/[A-Za-z0-9_./-]+\.wasm/g) ?? [])];
      for (const ref of wasmRefs) {
        const wasmHref = new URL(ref, httpHref).href;
        if (isUnderBase(wasmHref, base.href)) {
          await materialize(wasmHref);
        }
      }
    }
    return filePath;
  }

  // --- hooks ------------------------------------------------------------

  const hooks = registerHooks({
    async resolve(specifier, context, nextResolve) {
      const parentURL = context.parentURL ?? null;

      // Case A: specifier absolut http di bawah base yang dikonfigurasi.
      if (specifier.startsWith('http:') || specifier.startsWith('https:')) {
        if (isUnderBase(specifier, base.href)) {
          const filePath = await materialize(specifier);
          return { url: pathToFileURL(filePath).href, shortCircuit: true };
        }
        return nextResolve(specifier, context);
      }

      // Case B: specifier relatif dari modul yang sudah dimaterialisasi
      // (parent file: di dalam tempDir) -> terjemahkan ke http URL server.
      const isRelative = specifier.startsWith('./') || specifier.startsWith('../');
      if (isRelative && parentURL && parentURL.startsWith('file:')) {
        const parentPath = fileURLToPath(parentURL);
        const parentHref = tempFilePathToHttpHref(parentPath);
        if (parentHref !== null) {
          const httpHref = new URL(specifier, parentHref).href;
          if (isUnderBase(httpHref, base.href)) {
            const filePath = await materialize(httpHref);
            return { url: pathToFileURL(filePath).href, shortCircuit: true };
          }
          throw new Error(`http-bundle-loader: specifier '${specifier}' escapes served base ${basePath}`);
        }
      }

      return nextResolve(specifier, context);
    },

    async load(url, context, nextLoad) {
      if (url.startsWith('file:')) {
        const filePath = fileURLToPath(url);
        const root = path.resolve(tempDir);
        if (path.resolve(filePath).startsWith(root + path.sep)) {
          const source = await fs.promises.readFile(filePath, 'utf8');
          return { format: 'module', source, shortCircuit: true };
        }
      }
      return nextLoad(url, context);
    },
  });

  return {
    registry: { fetches },
    deregister: () => hooks.deregister(),
  };
}
