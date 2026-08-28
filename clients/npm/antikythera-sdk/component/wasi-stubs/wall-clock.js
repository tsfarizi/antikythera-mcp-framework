export function now() {
  const ms = Date.now();
  return { seconds: BigInt(Math.floor(ms / 1000)), nanoseconds: (ms % 1000) * 1_000_000 };
}
