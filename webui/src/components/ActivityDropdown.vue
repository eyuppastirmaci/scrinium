<script setup lang="ts">
import { ref, computed } from 'vue'
import { Activity } from '@lucide/vue'
import { useProcessingStatus } from '../composables/useProcessingStatus'

const { processingStatus } = useProcessingStatus()
const open = ref(false)

const entries = computed(() => {
  return Object.entries(processingStatus).map(([id, p]) => ({
    documentId: id,
    step: p.step,
    progress: p.progress,
  }))
})

const activeCount = computed(() => entries.value.length)

function toggle() {
  open.value = !open.value
}

function stepLabel(step: string): string {
  const labels: Record<string, string> = {
    received: 'Starting...',
    extracting_text: 'Extracting text',
    preprocessing_image: 'Preprocessing',
    running_ocr: 'Running OCR',
    extracting_metadata: 'Reading metadata',
    generating_thumbnail: 'Generating preview',
  }
  return labels[step] ?? step
}
</script>

<template>
  <div class="activity" v-if="activeCount > 0">
    <button class="activity__toggle" @click="toggle">
      <Activity :size="16" :stroke-width="1.5" />
      <span class="activity__badge">{{ activeCount }}</span>
    </button>

    <div v-if="open" class="activity__dropdown">
      <div class="activity__header">Processing</div>
      <div
        v-for="entry in entries"
        :key="entry.documentId"
        class="activity__item"
      >
        <div class="activity__info">
          <span class="activity__doc-id">{{ entry.documentId.slice(0, 8) }}...</span>
          <span class="activity__step">{{ stepLabel(entry.step) }}</span>
        </div>
        <div class="activity__bar-track">
          <div
            class="activity__bar-fill"
            :style="{ width: `${Math.max(entry.progress, 0)}%` }"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.activity {
  position: relative;
}

.activity__toggle {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: var(--radius-md);
  color: var(--color-accent);
  transition: background-color 0.15s;
}

.activity__toggle:hover {
  background-color: var(--color-bg-elevated);
}

.activity__badge {
  position: absolute;
  top: 2px;
  right: 2px;
  min-width: 14px;
  height: 14px;
  font-size: 9px;
  font-weight: 700;
  line-height: 14px;
  text-align: center;
  color: #fff;
  background-color: var(--color-accent);
  border-radius: var(--radius-pill);
}

.activity__dropdown {
  position: absolute;
  top: 42px;
  right: 0;
  width: 300px;
  background-color: var(--color-bg-surface);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  z-index: 100;
  overflow: hidden;
}

.activity__header {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  color: var(--color-text-tertiary);
  padding: 10px 14px 6px;
}

.activity__item {
  padding: 8px 14px;
  border-bottom: 1px solid var(--color-border-subtle);
}

.activity__item:last-child {
  border-bottom: none;
}

.activity__info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 6px;
}

.activity__doc-id {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-primary);
  font-family: var(--font-mono);
}

.activity__step {
  font-size: 11px;
  color: var(--color-text-secondary);
}

.activity__bar-track {
  height: 4px;
  background-color: var(--color-bg-base);
  border-radius: 2px;
  overflow: hidden;
}

.activity__bar-fill {
  height: 100%;
  background-color: var(--color-accent);
  border-radius: 2px;
  transition: width 0.6s ease;
}
</style>
