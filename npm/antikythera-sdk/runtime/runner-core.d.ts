/**
 * Client-core runtime: loads the jco-transpiled component and owns the tool
 * loop (K1).
 */

import {
  AgentRuntimeOptionsBase,
  ClientCoreRuntime,
  LlmOptions,
} from './index';

export interface ClientCoreOptions extends AgentRuntimeOptionsBase {
  core?: 'client';
  llm?: LlmOptions;
}

export function createClientCoreRuntime(options: ClientCoreOptions): Promise<ClientCoreRuntime>;

/** Load the jco-transpiled component runner namespace (cached per module). */
export function loadRunnerModule(): Promise<Record<string, (...args: any[]) => any>>;
