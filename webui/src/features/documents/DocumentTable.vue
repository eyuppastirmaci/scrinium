<script setup lang="ts">
import DocumentRow from './DocumentRow.vue'
import type { DocumentSummary } from '../../api/documents'

defineProps<{ documents: DocumentSummary[] }>()
</script>

<template>
  <div class="table-wrap">
    <div class="table-header">
      <span class="table-header__name">Name</span>
      <span class="table-header__cell">Size</span>
      <span class="table-header__cell">Date</span>
      <span class="table-header__cell">Status</span>
    </div>
    <DocumentRow v-for="doc in documents" :key="doc.id" :document="doc" />
  </div>
</template>

<style scoped>
.table-wrap {
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-surface);
  overflow: hidden;
}

.table-header {
  display: grid;
  grid-template-columns: 1fr 100px 140px 100px;
  gap: 12px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--color-border-default);
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}

.table-header__name {
  min-width: 0;
}

.table-header__cell {
  /* alignment matches row cells */
}
</style>
