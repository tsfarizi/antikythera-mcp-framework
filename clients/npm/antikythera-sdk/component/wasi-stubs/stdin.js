import { InputStream } from './streams.js';

export function getStdin() {
  return new InputStream();
}
