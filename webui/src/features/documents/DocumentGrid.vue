<script setup lang="ts">
import DocumentCard from './DocumentCard.vue'
import type { DocumentSummary } from '../../api/documents'

defineProps<{ documents: DocumentSummary[] }>()
</script>

<template>
  <div class="grid">
    <DocumentCard v-for="doc in documents" :key="doc.id" :document="doc" />
  </div>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 16px;
}
</style>
