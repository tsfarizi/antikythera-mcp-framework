<script setup lang="ts">
import { ref } from 'vue'
import { eventBus } from '@/shared/bus/event-bus'

interface Session {
  id: string
  title: string
  createdAt: number
}

const sessions = ref<Session[]>([])
const activeSessionId = ref<string | null>(null)

// F6: Only listen for sessions created by WASM, don't create independently
eventBus.on('session:created', ({ sessionId, title }) => {
  if (!sessions.value.find((s) => s.id === sessionId)) {
    sessions.value.unshift({ id: sessionId, title, createdAt: Date.now() })
  }
  activeSessionId.value = sessionId
})

eventBus.on('session:switched', ({ sessionId }) => {
  activeSessionId.value = sessionId
})

function getActiveSessionId() {
  return activeSessionId.value
}

defineExpose({ getActiveSessionId })
</script>

<template>
  <div class="session-sidebar">
    <div class="sidebar-header">
      <h3>Antikythera</h3>
    </div>

    <div class="session-list">
      <div
        v-for="session in sessions"
        :key="session.id"
        :class="['session-item', { active: session.id === activeSessionId }]"
      >
        <span class="session-title">{{ session.title }}</span>
      </div>

      <div v-if="sessions.length === 0" class="empty-sessions">
        Start a conversation below
      </div>
    </div>
  </div>
</template>

<style scoped>
.session-sidebar {
  width: 260px;
  background: #1a1a2e;
  color: white;
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.sidebar-header {
  padding: 16px;
  border-bottom: 1px solid rgba(255,255,255,0.1);
}

.sidebar-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.session-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.session-item {
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  margin-bottom: 4px;
  font-size: 14px;
  transition: background 0.2s;
}

.session-item:hover {
  background: rgba(255,255,255,0.1);
}

.session-item.active {
  background: rgba(59,130,246,0.5);
}

.empty-sessions {
  padding: 20px;
  text-align: center;
  color: rgba(255,255,255,0.4);
  font-size: 13px;
}
</style>
