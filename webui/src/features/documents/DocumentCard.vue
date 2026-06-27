<script setup lang="ts">
import { ref } from 'vue'
import { FileText, Image } from '@lucide/vue'
import StatusBadge from '../../components/StatusBadge.vue'
import { formatFileSize, formatDate } from '../../utils/format'
import { getThumbnailUrl, type DocumentSummary } from '../../api/documents'

const props = defineProps<{ document: DocumentSummary }>()

const thumbFailed = ref(false)

function isImage(contentType: string): boolean {
  return contentType.startsWith('image/')
}

const showThumbnail = props.document.status === 'READY'
</script>

<template>
  <RouterLink
    :to="{ name: 'document-detail', params: { id: document.id } }"
    class="card"
  >
    <div class="card__top">
      <img
        v-if="showThumbnail && !thumbFailed"
        :src="getThumbnailUrl(document.id, 'small')"
        alt=""
        class="card__thumb"
        @error="thumbFailed = true"
      />
      <component
        v-else
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

.card__thumb {
  width: 40px;
  height: 40px;
  object-fit: cover;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border-subtle);
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
