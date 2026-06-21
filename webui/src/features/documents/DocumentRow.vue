<script setup lang="ts">
import { FileText, Image } from '@lucide/vue'
import StatusBadge from '../../components/StatusBadge.vue'
import { formatFileSize, formatDate } from '../../utils/format'
import type { DocumentSummary } from '../../api/documents'

defineProps<{ document: DocumentSummary }>()

function isImage(contentType: string): boolean {
  return contentType.startsWith('image/')
}
</script>

<template>
  <RouterLink
    :to="{ name: 'document-detail', params: { id: document.id } }"
    class="row"
  >
    <div class="row__name">
      <component
        :is="isImage(document.contentType) ? Image : FileText"
        :size="16" :stroke-width="1.5"
        class="row__icon"
      />
      <span class="row__filename">{{ document.fileName }}</span>
    </div>
    <span class="row__cell">{{ formatFileSize(document.sizeBytes) }}</span>
    <span class="row__cell">{{ formatDate(document.createdAt) }}</span>
    <span class="row__cell"><StatusBadge :status="document.status" /></span>
  </RouterLink>
</template>

<style scoped>
.row {
  display: grid;
  grid-template-columns: 1fr 100px 140px 100px;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--color-border-subtle);
  transition: background-color 0.15s;
}

.row:last-child {
  border-bottom: none;
}

.row:hover {
  background-color: var(--color-bg-elevated);
}

.row__name {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.row__icon {
  color: var(--color-accent);
  flex-shrink: 0;
}

.row__filename {
  font-size: 13px;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row__cell {
  font-size: 12px;
  color: var(--color-text-secondary);
}
</style>
