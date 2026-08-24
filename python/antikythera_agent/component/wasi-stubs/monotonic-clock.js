export function now() {
  if (typeof performance !== 'undefined' && typeof performance.now === 'function') {
    return BigInt(Math.floor(performance.now() * 1_000_000));
  }
  return BigInt(Date.now()) * 1_000_000n;
}
