<script setup lang="ts">
import { ref } from 'vue'
import { CloudUpload } from '@lucide/vue'

defineProps<{
  maxFileSizeLabel: string
}>()

const emit = defineEmits<{
  files: [files: File[]]
}>()

const dragging = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

function onDrop(e: DragEvent) {
  dragging.value = false
  if (!e.dataTransfer?.files.length) return
  emit('files', Array.from(e.dataTransfer.files))
}

function onDragOver(e: DragEvent) {
  e.preventDefault()
  dragging.value = true
}

function onDragLeave() {
  dragging.value = false
}

function openPicker() {
  fileInput.value?.click()
}

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (!input.files?.length) return
  emit('files', Array.from(input.files))
  input.value = ''
}
</script>

<template>
  <div
    class="dropzone"
    :class="{ 'dropzone--active': dragging }"
    @drop.prevent="onDrop"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @click="openPicker"
  >
    <input
      ref="fileInput"
      type="file"
      multiple
      class="dropzone__input"
      @change="onFileChange"
    />
    <div class="dropzone__icon">
      <CloudUpload :size="28" :stroke-width="1.5" />
    </div>
    <p class="dropzone__title">Drop files here or click to browse</p>
    <p class="dropzone__subtitle">PDF, JPG, PNG, WebP, TIFF — max {{ maxFileSizeLabel }} per file</p>
  </div>
</template>

<style scoped>
.dropzone {
  border: 2px dashed var(--color-border-default);
  border-radius: var(--radius-lg);
  padding: 48px 24px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.15s, background-color 0.15s;
}

.dropzone:hover,
.dropzone--active {
  border-color: var(--color-accent);
  background-color: var(--color-accent-subtle);
}

.dropzone__input {
  display: none;
}

.dropzone__icon {
  width: 48px;
  height: 48px;
  margin: 0 auto 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  background-color: var(--color-accent-subtle);
  color: var(--color-accent);
}

.dropzone__title {
  font-size: 15px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: 4px;
}

.dropzone__subtitle {
  font-size: 13px;
  color: var(--color-text-secondary);
}
</style>
