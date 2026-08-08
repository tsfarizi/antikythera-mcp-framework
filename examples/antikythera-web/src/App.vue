<script setup lang="ts">
import { onMounted } from 'vue'
import { initWasm } from './shared/wasm'
import ChatView from './slices/chat/components/ChatView.vue'
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
  <main class="main-content">
    <ChatView />
    <SettingsPanel />
  </main>
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

.main-content {
  flex: 1;
  height: 100vh;
  position: relative;
  overflow: hidden;
}
</style>
