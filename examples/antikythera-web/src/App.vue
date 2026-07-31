<script setup lang="ts">
import { onMounted } from 'vue'
import { initWasm } from './shared/wasm'
import ChatView from './slices/chat/components/ChatView.vue'
import SessionSidebar from './slices/session/components/SessionSidebar.vue'
import SettingsPanel from './slices/settings/components/SettingsPanel.vue'

onMounted(async () => {
  try {
    await initWasm()
    console.log('WASM initialized successfully')
  } catch (e) {
    console.error('Failed to initialize WASM:', e)
  }
})
</script>

<template>
  <div class="app-layout">
    <SessionSidebar />
    <main class="main-content">
      <ChatView />
      <SettingsPanel />
    </main>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
}

.app-layout {
  display: flex;
  height: 100vh;
}

.main-content {
  flex: 1;
  position: relative;
}
</style>
