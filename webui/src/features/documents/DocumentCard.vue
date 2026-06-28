<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { FileText, Image } from '@lucide/vue'
import StatusBadge from '../../components/StatusBadge.vue'
import ProcessingIndicator from '../../components/ProcessingIndicator.vue'
import { formatFileSize, formatDate } from '../../utils/format'
import { getThumbnailUrl, type DocumentSummary } from '../../api/documents'
import { useProcessingStatus } from '../../composables/useProcessingStatus'

const props = defineProps<{ document: DocumentSummary }>()

const { processingStatus, completedIds, failedIds } = useProcessingStatus()

const thumbFailed = ref(false)
const thumbLoaded = ref(false)
const thumbVersion = ref(0)
let thumbRetries = 0

function isImage(contentType: string): boolean {
  return contentType.startsWith('image/')
}

const effectiveStatus = computed(() => {
  if (completedIds.has(props.document.id)) return 'READY' as const
  if (failedIds.has(props.document.id)) return 'FAILED' as const
  return props.document.status
})

watch(effectiveStatus, (status) => {
  if (status === 'READY') {
    thumbFailed.value = false
    thumbLoaded.value = false
    thumbRetries = 0
    thumbVersion.value = Date.now()
  }
})

const thumbnailUrl = computed(() => {
  const base = getThumbnailUrl(props.document.id, 'small')
  return thumbVersion.value ? `${base}&v=${thumbVersion.value}` : base
})

function onThumbLoad() {
  thumbLoaded.value = true
}

function onThumbError() {
  if (completedIds.has(props.document.id) && thumbRetries < 3) {
    thumbRetries++
    setTimeout(() => { thumbVersion.value = Date.now() }, 1000 * thumbRetries)
  } else {
    thumbFailed.value = true
  }
}

const showSkeleton = computed(() => showThumbnail.value && !thumbFailed.value && !thumbLoaded.value)

const progress = computed(() => processingStatus[props.document.id])
const showThumbnail = computed(() => effectiveStatus.value === 'READY' && !progress.value)
const showIndicator = computed(() => effectiveStatus.value === 'PENDING' || !!progress.value)
</script>

<template>
  <RouterLink
    :to="{ name: 'document-detail', params: { id: document.id } }"
    class="card"
  >
    <div class="card__top">
      <ProcessingIndicator
        v-if="showIndicator"
        :status="effectiveStatus"
        :progress="progress"
        :size="36"
      />
      <div v-else-if="showThumbnail && !thumbFailed" class="card__thumb-wrap">
        <div v-if="showSkeleton" class="card__skeleton" />
        <img
          :src="thumbnailUrl"
          alt=""
          class="card__thumb"
          :class="{ 'card__thumb--loaded': thumbLoaded }"
          @load="onThumbLoad"
          @error="onThumbError"
        />
      </div>
      <component
        v-else-if="!showIndicator"
        :is="isImage(document.contentType) ? Image : FileText"
        :size="28" :stroke-width="1.5"
        class="card__icon"
      />
      <StatusBadge :status="effectiveStatus" />
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

.card__thumb-wrap {
  width: 40px;
  height: 40px;
  position: relative;
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.card__skeleton {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    90deg,
    var(--color-bg-elevated) 25%,
    var(--color-border-subtle) 50%,
    var(--color-bg-elevated) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: var(--radius-sm);
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.card__thumb {
  width: 40px;
  height: 40px;
  object-fit: cover;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border-subtle);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.card__thumb--loaded {
  opacity: 1;
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
