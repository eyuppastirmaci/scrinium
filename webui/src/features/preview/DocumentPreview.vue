<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
import ImagePreview from './ImagePreview.vue'

const PdfPreview = defineAsyncComponent(() => import('./PdfPreview.vue'))

const props = defineProps<{
  contentType: string
  previewUrl: string
}>()

const isPdf = props.contentType === 'application/pdf'
const isImage = props.contentType.startsWith('image/')
</script>

<template>
  <div class="doc-preview">
    <PdfPreview v-if="isPdf" :url="previewUrl" />
    <ImagePreview v-else-if="isImage" :url="previewUrl" />
    <div v-else class="doc-preview__unsupported">
      Preview is not available for this file type. Use the download button to view the file.
    </div>
  </div>
</template>

<style scoped>
.doc-preview__unsupported {
  text-align: center;
  padding: 48px 0;
  color: var(--color-text-tertiary);
  font-size: 13px;
}
</style>
