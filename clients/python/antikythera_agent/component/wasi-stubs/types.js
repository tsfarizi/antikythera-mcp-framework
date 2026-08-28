export class Descriptor {
  writeViaStream(offset) {
    throw new Error('Descriptor.writeViaStream: no preopened filesystem descriptor available');
  }

  appendViaStream() {
    throw new Error('Descriptor.appendViaStream: no preopened filesystem descriptor available');
  }

  getType() {
    return 'unknown';
  }

  stat() {
    return { type: 'unknown', linkCount: 0n, size: 0n };
  }
}

export function filesystemErrorCode(err) {
  return undefined;
}
