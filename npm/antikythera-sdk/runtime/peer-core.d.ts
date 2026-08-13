/**
 * Server-core runtime: control-channel peer without the WASM runner.
 */

import { AgentRuntimeOptionsBase, ServerCoreRuntime } from './index';

export interface ServerCoreOptions extends AgentRuntimeOptionsBase {
  core: 'server';
}

export function createServerCoreRuntime(options: ServerCoreOptions): Promise<ServerCoreRuntime>;
