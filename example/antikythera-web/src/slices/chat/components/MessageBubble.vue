<script setup lang="ts">
import { ref } from 'vue'
import type { Message } from '../core/message'

defineProps<{
  message: Message
}>()

const toolsExpanded = ref(false)

function formatJson(obj: unknown): string {
  if (obj === null || obj === undefined) return 'null'
  return JSON.stringify(obj, null, 2)
}

function toolNameDisplay(tool: string): string {
  return tool.replace(/_/g, ' ')
}
</script>

<template>
  <div :class="['message', message.role]">
    <!-- Tool calls (collapsible) -->
    <div v-if="message.toolCalls && message.toolCalls.length > 0" class="tool-calls">
      <button class="tool-toggle" @click="toolsExpanded = !toolsExpanded">
        <span class="tool-icon">⚙</span>
        <span class="tool-label">
          {{ message.toolCalls.length }} tool call{{ message.toolCalls.length > 1 ? 's' : '' }}
        </span>
        <span class="tool-chevron" :class="{ expanded: toolsExpanded }">▸</span>
      </button>
      <div v-if="toolsExpanded" class="tool-details">
        <div
          v-for="(tc, idx) in message.toolCalls"
          :key="idx"
          class="tool-entry"
          :class="{ 'tool-error': !tc.success }"
        >
          <div class="tool-header">
            <span class="tool-status">{{ tc.success ? '✓' : '✗' }}</span>
            <span class="tool-name">{{ toolNameDisplay(tc.tool) }}</span>
          </div>
          <div class="tool-io">
            <div class="tool-section">
              <span class="tool-section-label">Input</span>
              <pre class="tool-json">{{ formatJson(tc.input) }}</pre>
            </div>
            <div class="tool-section">
              <span class="tool-section-label">{{ tc.success ? 'Output' : 'Error' }}</span>
              <pre v-if="tc.success" class="tool-json">{{ formatJson(tc.output) }}</pre>
              <pre v-else class="tool-json tool-error-text">{{ tc.error || 'Unknown error' }}</pre>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Message content -->
    <div v-if="message.content" class="bubble">
      {{ message.content }}
    </div>
  </div>
</template>

<style scoped>
.message {
  display: flex;
  flex-direction: column;
}

.message.user {
  align-items: flex-end;
}

.message.assistant {
  align-items: flex-start;
}

.bubble {
  max-width: 70%;
  padding: 12px 16px;
  border-radius: 16px;
  font-size: 15px;
  line-height: 1.5;
  white-space: pre-wrap;
}

.message.user .bubble {
  background: #3b82f6;
  color: white;
  border-bottom-right-radius: 4px;
}

.message.assistant .bubble {
  background: #f3f4f6;
  color: #111;
  border-bottom-left-radius: 4px;
}

/* Tool calls collapsible */
.tool-calls {
  max-width: 70%;
  margin-bottom: 6px;
}

.tool-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: #f0f4ff;
  border: 1px solid #d0daf7;
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  color: #4b5563;
  transition: background 0.15s;
}

.tool-toggle:hover {
  background: #e5edff;
}

.tool-icon {
  font-size: 14px;
}

.tool-label {
  font-weight: 500;
}

.tool-chevron {
  transition: transform 0.2s;
  font-size: 12px;
}

.tool-chevron.expanded {
  transform: rotate(90deg);
}

.tool-details {
  margin-top: 4px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.tool-entry {
  background: #fafbfc;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 8px 12px;
  font-size: 13px;
}

.tool-entry.tool-error {
  border-color: #fca5a5;
  background: #fef2f2;
}

.tool-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}

.tool-status {
  font-size: 12px;
}

.tool-entry:not(.tool-error) .tool-status {
  color: #22c55e;
}

.tool-error .tool-status {
  color: #dc2626;
}

.tool-name {
  font-weight: 600;
  color: #374151;
}

.tool-io {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tool-section {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.tool-section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: #9ca3af;
  letter-spacing: 0.5px;
}

.tool-json {
  background: #1e1e2e;
  color: #cdd6f4;
  padding: 6px 10px;
  border-radius: 6px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  line-height: 1.4;
  overflow-x: auto;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

.tool-error-text {
  background: #fef2f2;
  color: #dc2626;
}

/* Cursor animation */
.cursor {
  animation: blink 1s step-end infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}
</style>
