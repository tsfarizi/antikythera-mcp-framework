export function initialize(data) {
  globalThis.__E2E_LOADER_INIT_DATA__ = data;
  const { port } = arguments[1] ?? {};
  if (port) { port.on('message', (m) => { globalThis.__E2E_LOADER_PORT_MSG__ = m; }); port.start(); }
}
export async function resolve(specifier, context, nextResolve) {
  globalThis.__E2E_LOADER_RESOLVE_CALLS__ = (globalThis.__E2E_LOADER_RESOLVE_CALLS__ || 0) + 1;
  return nextResolve(specifier, context);
}
