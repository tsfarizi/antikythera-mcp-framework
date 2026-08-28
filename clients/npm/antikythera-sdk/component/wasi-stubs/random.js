export function getRandomBytes(len) {
  const length = Number(len);
  const bytes = new Uint8Array(length);
  if (typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function') {
    const CHUNK = 65536;
    for (let offset = 0; offset < length; offset += CHUNK) {
      crypto.getRandomValues(bytes.subarray(offset, Math.min(offset + CHUNK, length)));
    }
  } else {
    for (let i = 0; i < length; i++) bytes[i] = Math.floor(Math.random() * 256);
  }
  return bytes;
}
