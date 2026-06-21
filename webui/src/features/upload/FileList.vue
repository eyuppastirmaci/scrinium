<script setup lang="ts">
import { computed, watch, nextTick } from 'vue'
import { FileText, Image, X, Upload, AlertTriangle } from '@lucide/vue'
import UploadProgress from './UploadProgress.vue'
import { formatFileSize } from '../../utils/format'
import type { StagedFile } from './types'

const props = defineProps<{
  files: StagedFile[]
}>()

const emit = defineEmits<{
  remove: [index: number]
  upload: []
  clear: []
}>()

const totalSize = computed(() =>
  props.files.reduce((sum, f) => sum + f.file.size, 0)
)

const isIdle = computed(() =>
  props.files.every((f) => f.status === 'idle' || f.status === 'rejected')
)

const uploadableCount = computed(() =>
  props.files.filter((f) => f.status === 'idle').length
)

function isPdf(file: File): boolean {
  return file.type === 'application/pdf'
}

function scheduleRejectDismiss(index: number) {
  setTimeout(() => {
    const el = document.querySelector(`[data-file-index="${index}"]`) as HTMLElement | null
    if (!el) return

    el.classList.add('file-item--dismissing')
    el.addEventListener('transitionend', () => {
      emit('remove', index)
    }, { once: true })
  }, 5000)
}

watch(() => props.files.length, async () => {
  await nextTick()
  props.files.forEach((f, i) => {
    if (f.status === 'rejected') {
      const el = document.querySelector(`[data-file-index="${i}"]`) as HTMLElement | null
      if (el && !el.dataset.dismissScheduled) {
        el.dataset.dismissScheduled = '1'
        scheduleRejectDismiss(i)
      }
    }
  })
}, { immediate: true })
</script>

<template>
  <div class="file-list">
    <div class="file-list__header">
      <span class="file-list__summary">
        {{ files.length }} file{{ files.length > 1 ? 's' : '' }} · {{ formatFileSize(totalSize) }}
      </span>
      <button v-if="isIdle" class="file-list__clear" @click="emit('clear')">Clear all</button>
    </div>

    <ul class="file-list__items">
      <li
        v-for="(staged, index) in files"
        :key="index"
        :data-file-index="index"
        class="file-item"
        :class="{ 'file-item--rejected': staged.status === 'rejected' }"
      >
        <AlertTriangle
          v-if="staged.status === 'rejected'"
          :size="18" :stroke-width="1.5"
          class="file-item__icon file-item__icon--rejected"
        />
        <component
          v-else
          :is="isPdf(staged.file) ? FileText : Image"
          :size="18" :stroke-width="1.5"
          class="file-item__icon"
        />

        <div class="file-item__info">
          <span
            class="file-item__name"
            :class="{ 'file-item__name--rejected': staged.status === 'rejected' }"
          >
            {{ staged.file.name }}
          </span>
          <span v-if="staged.status === 'rejected'" class="file-item__reject-reason">
            {{ staged.rejectReason }}
          </span>
          <span v-else-if="staged.status === 'error'" class="file-item__error">
            {{ staged.error }}
            <RouterLink
              v-if="staged.duplicateId"
              :to="{ name: 'document-detail', params: { id: staged.duplicateId } }"
              class="file-item__dup-link"
            >
              View existing
            </RouterLink>
          </span>
          <span v-else class="file-item__meta">{{ formatFileSize(staged.file.size) }}</span>
        </div>

        <UploadProgress
          v-if="staged.status === 'uploading' || staged.status === 'done' || staged.status === 'error'"
          :percent="staged.progress"
          :status="staged.status"
          :size="42"
        />
        <button
          v-else
          class="file-item__remove"
          @click="emit('remove', index)"
        >
          <X :size="14" :stroke-width="2" />
        </button>
      </li>
    </ul>

    <button
      v-if="isIdle && uploadableCount > 0"
      class="file-list__upload"
      @click="emit('upload')"
    >
      <Upload :size="16" :stroke-width="2" />
      Upload {{ uploadableCount }} file{{ uploadableCount > 1 ? 's' : '' }}
    </button>
  </div>
</template>

<style scoped>
.file-list {
  margin-top: 20px;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-surface);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
  flex: 1;
}

.file-list__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

.file-list__summary {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.file-list__clear {
  font-size: 12px;
  color: var(--color-text-secondary);
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  transition: color 0.15s, background-color 0.15s;
}

.file-list__clear:hover {
  color: var(--color-danger);
  background-color: var(--color-danger-subtle);
}

.file-list__items {
  list-style: none;
  flex: 1;
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--color-border-subtle);
  transition: transform 1s ease, opacity 1s ease, max-height 0.4s ease 1s, padding 0.4s ease 1s, border-bottom-width 0.4s ease 1s;
  transform: translateX(0);
  opacity: 1;
  max-height: 80px;
  overflow: hidden;
}

.file-item:last-child {
  border-bottom: none;
}

.file-item--rejected {
  background-color: var(--color-danger-subtle);
  opacity: 0.7;
}

.file-item--dismissing {
  transform: translateX(100%);
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
  border-bottom-width: 0;
}

.file-item__icon {
  color: var(--color-accent);
  flex-shrink: 0;
}

.file-item__icon--rejected {
  color: var(--color-danger);
}

.file-item__info {
  flex: 1;
  min-width: 0;
}

.file-item__name {
  display: block;
  font-size: 13px;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-item__name--rejected {
  color: var(--color-danger);
}

.file-item__meta {
  font-size: 11px;
  color: var(--color-text-secondary);
}

.file-item__reject-reason {
  display: block;
  font-size: 11px;
  color: var(--color-danger);
}

.file-item__error {
  font-size: 11px;
  color: var(--color-danger);
}

.file-item__dup-link {
  color: var(--color-accent);
  margin-left: 6px;
  text-decoration: underline;
}

.file-item__remove {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  color: var(--color-text-tertiary);
  transition: color 0.15s, background-color 0.15s;
}

.file-item__remove:hover {
  color: var(--color-danger);
  background-color: var(--color-danger-subtle);
}

.file-list__upload {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 12px;
  font-size: 14px;
  font-weight: 500;
  color: #fff;
  background-color: var(--color-accent);
  transition: background-color 0.15s;
  flex-shrink: 0;
}

.file-list__upload:hover {
  background-color: var(--color-accent-hover);
}
</style>
