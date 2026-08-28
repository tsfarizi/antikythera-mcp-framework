<script setup lang="ts">
import ChatView from './slices/chat/components/ChatView.vue'
import SettingsPanel from './slices/settings/components/SettingsPanel.vue'
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
