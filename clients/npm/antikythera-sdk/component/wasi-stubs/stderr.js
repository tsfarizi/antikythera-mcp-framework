import { OutputStream } from './streams.js';

function makeSink(channel) {
  return (contents) => {
    if (typeof process !== 'undefined' && process[channel] && typeof process[channel].write === 'function') {
      process[channel].write(new Uint8Array(contents));
    } else if (typeof console !== 'undefined') {
      const text = new TextDecoder().decode(contents);
      if (channel === 'stderr') console.error(text);
      else console.log(text);
    }
  };
}

export function getStderr() {
  return new OutputStream(makeSink('stderr'));
}
