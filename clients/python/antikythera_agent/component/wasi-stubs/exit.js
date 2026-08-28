export function exit(status) {
  throw Object.assign(new Error('WASI exit requested: ' + JSON.stringify(status)), { wasiExit: true });
}
