<script setup lang="ts">
import { ref } from 'vue'

defineProps<{ url: string }>()

const loading = ref(true)
const error = ref(false)

function onLoad() {
  loading.value = false
}

function onError() {
  loading.value = false
  error.value = true
}
</script>

<template>
  <div class="image-preview">
    <div v-if="loading && !error" class="image-preview__status">Loading image...</div>

    <div v-if="error" class="image-preview__error">
      Failed to load image preview.
    </div>

    <img
      v-show="!loading && !error"
      :src="url"
      alt="Document preview"
      class="image-preview__img"
      @load="onLoad"
      @error="onError"
    />
  </div>
</template>

<style scoped>
.image-preview {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.image-preview__status {
  text-align: center;
  padding: 48px 0;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.image-preview__error {
  text-align: center;
  padding: 48px 0;
  color: var(--color-danger);
  font-size: 13px;
}

.image-preview__img {
  max-width: 100%;
  max-height: 600px;
  object-fit: contain;
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
}
</style>
