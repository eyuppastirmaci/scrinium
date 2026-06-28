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
    class="row"
  >
    <div class="row__name">
      <ProcessingIndicator
        v-if="showIndicator"
        :status="effectiveStatus"
        :progress="progress"
        :size="24"
      />
      <div v-else-if="showThumbnail && !thumbFailed" class="row__thumb-wrap">
        <div v-if="showSkeleton" class="row__skeleton" />
        <img
          :src="thumbnailUrl"
          alt=""
          class="row__thumb"
          :class="{ 'row__thumb--loaded': thumbLoaded }"
          @load="onThumbLoad"
          @error="onThumbError"
        />
      </div>
      <component
        v-else-if="!showIndicator"
        :is="isImage(document.contentType) ? Image : FileText"
        :size="16" :stroke-width="1.5"
        class="row__icon"
      />
      <span class="row__filename">{{ document.fileName }}</span>
    </div>
    <span class="row__cell">{{ formatFileSize(document.sizeBytes) }}</span>
    <span class="row__cell">{{ formatDate(document.createdAt) }}</span>
    <span class="row__cell"><StatusBadge :status="effectiveStatus" /></span>
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

.row__thumb-wrap {
  width: 24px;
  height: 24px;
  position: relative;
  border-radius: 3px;
  overflow: hidden;
  flex-shrink: 0;
}

.row__skeleton {
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
  border-radius: 3px;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.row__thumb {
  width: 24px;
  height: 24px;
  object-fit: cover;
  border-radius: 3px;
  border: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.row__thumb--loaded {
  opacity: 1;
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
