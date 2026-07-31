<script setup lang="ts">
import { ref } from 'vue'
import { getOllamaModel, setOllamaModel } from '@/shared/adapters/llm-adapter'

const isOpen = ref(false)
const model = ref(getOllamaModel())

const popularModels = [
  'gpt-oss:120b-cloud',
  'llama3.1',
  'llama3',
  'mistral',
  'mixtral',
  'gemma2',
  'phi3',
  'qwen2.5',
  'deepseek-coder-v2',
]

function save() {
  setOllamaModel(model.value)
  isOpen.value = false
}
</script>

<template>
  <button class="settings-toggle" @click="isOpen = !isOpen">⚙️</button>

  <div v-if="isOpen" class="settings-overlay" @click.self="isOpen = false">
    <div class="settings-panel">
      <h3>Settings</h3>
      <p class="subtitle">Ollama must be running at localhost:11434</p>

      <label>
        Model
        <select v-model="model">
          <option v-for="m in popularModels" :key="m" :value="m">{{ m }}</option>
        </select>
      </label>

      <label>
        Or type custom model name
        <input v-model="model" type="text" placeholder="model-name" />
      </label>

      <div class="settings-actions">
        <button class="btn-cancel" @click="isOpen = false">Cancel</button>
        <button class="btn-save" @click="save">Save</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-toggle {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 100;
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  padding: 8px;
  border-radius: 8px;
}

.settings-toggle:hover {
  background: rgba(0,0,0,0.05);
}

.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.settings-panel {
  background: white;
  padding: 24px;
  border-radius: 12px;
  width: 360px;
  box-shadow: 0 20px 60px rgba(0,0,0,0.2);
}

.settings-panel h3 {
  margin: 0 0 4px;
}

.subtitle {
  margin: 0 0 16px;
  font-size: 13px;
  color: #6b7280;
}

label {
  display: block;
  margin-bottom: 16px;
  font-size: 14px;
  color: #374151;
}

label input, label select {
  display: block;
  width: 100%;
  margin-top: 6px;
  padding: 10px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
}

.settings-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 20px;
}

.btn-cancel, .btn-save {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
}

.btn-cancel {
  background: #f3f4f6;
  border: 1px solid #d1d5db;
}

.btn-save {
  background: #3b82f6;
  color: white;
  border: none;
}
</style>
