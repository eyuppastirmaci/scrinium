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
    class="card"
  >
    <div class="card__top">
      <component
        :is="isImage(document.contentType) ? Image : FileText"
        :size="28" :stroke-width="1.5"
        class="card__icon"
      />
      <StatusBadge :status="document.status" />
    </div>
    <p class="card__name">{{ document.fileName }}</p>
    <p class="card__meta">{{ formatFileSize(document.sizeBytes) }} · {{ formatDate(document.createdAt) }}</p>
  </RouterLink>
</template>

<style scoped>
.card {
  display: flex;
  flex-direction: column;
  padding: 16px;
  background-color: var(--color-bg-surface);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  transition: border-color 0.15s;
}

.card:hover {
  border-color: var(--color-accent);
}

.card__top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 12px;
}

.card__icon {
  color: var(--color-accent);
}

.card__name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.card__meta {
  font-size: 11px;
  color: var(--color-text-secondary);
}
</style>
