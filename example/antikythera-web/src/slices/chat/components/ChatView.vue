<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import { useChat } from '../flow/send-message'
import MessageBubble from './MessageBubble.vue'

const { messages, status, streamingContent, error, sendMessage } = useChat()
const input = ref('')
const messagesContainer = ref<HTMLElement>()

async function handleSend() {
  const text = input.value.trim()
  if (!text || status.value === 'streaming') return
  input.value = ''
  await sendMessage(text)
  await nextTick()
  scrollToBottom()
}

function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

watch(messages, () => {
  nextTick(scrollToBottom)
}, { deep: true })
</script>

<template>
  <div class="chat-view">
    <div class="chat-header">
      <h2>Antikythera Chatbot</h2>
      <span v-if="status === 'streaming'" class="streaming-indicator">● Streaming...</span>
    </div>

    <div ref="messagesContainer" class="messages-container">
      <div v-if="messages.length === 0" class="empty-state">
        <p>Start a conversation by typing a message below.</p>
      </div>

      <MessageBubble
        v-for="msg in messages"
        :key="msg.id"
        :message="msg"
      />

      <div v-if="streamingContent" class="message assistant streaming">
        <div class="bubble">
          {{ streamingContent }}<span class="cursor">▊</span>
        </div>
      </div>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>

    <form class="input-form" @submit.prevent="handleSend">
      <input
        v-model="input"
        type="text"
        placeholder="Type your message..."
        :disabled="status === 'streaming'"
        autofocus
      />
      <button
        type="submit"
        :disabled="!input.trim() || status === 'streaming'"
      >
        {{ status === 'streaming' ? '...' : 'Send' }}
      </button>
    </form>
  </div>
</template>

<style scoped>
.chat-view {
  display: flex;
  flex-direction: column;
  height: 100vh;
  max-width: 800px;
  margin: 0 auto;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.chat-header {
  padding: 16px 20px;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  align-items: center;
  gap: 12px;
}

.chat-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.streaming-indicator {
  color: #22c55e;
  font-size: 14px;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #888;
}

.error-message {
  padding: 12px 16px;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 8px;
  color: #dc2626;
  font-size: 14px;
}

.input-form {
  display: flex;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid #e0e0e0;
  background: white;
}

.input-form input {
  flex: 1;
  padding: 12px 16px;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 15px;
  outline: none;
  transition: border-color 0.2s;
}

.input-form input:focus {
  border-color: #3b82f6;
}

.input-form input:disabled {
  background: #f3f4f6;
}

.input-form button {
  padding: 12px 24px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}

.input-form button:hover:not(:disabled) {
  background: #2563eb;
}

.input-form button:disabled {
  background: #9ca3af;
  cursor: not-allowed;
}

.streaming .bubble::after {
  content: '';
}
</style>
