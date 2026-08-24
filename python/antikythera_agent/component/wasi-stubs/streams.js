export class InputStream {
  read(len) {
    return new Uint8Array(0);
  }

  blockingRead(len) {
    return new Uint8Array(0);
  }

  skip(len) {
    return 0n;
  }

  subscribe() {
    throw new Error('InputStream.subscribe: not supported by antikythera wasi-stubs');
  }
}

export class OutputStream {
  constructor(sink) {
    this._sink = sink || null;
  }

  checkWrite() {
    return 65536n;
  }

  write(contents) {
    if (this._sink) this._sink(contents);
  }

  blockingWriteAndFlush(contents) {
    if (this._sink) this._sink(contents);
  }

  blockingFlush() {}

  flush() {}

  writeZeroes(len) {}

  blockingWriteZeroesAndFlush(len) {}

  splice(src, len) {
    return 0n;
  }

  blockingSplice(src, len) {
    return 0n;
  }

  subscribe() {
    throw new Error('OutputStream.subscribe: not supported by antikythera wasi-stubs');
  }
}
