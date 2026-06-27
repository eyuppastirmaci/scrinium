<script setup lang="ts">
import { computed } from 'vue'
import { Clock, Check, AlertCircle } from '@lucide/vue'
import type { DocumentStatus } from '../api/documents'
import type { ProcessingProgress } from '../composables/useProcessingStatus'

const props = defineProps<{
  status: DocumentStatus
  progress?: ProcessingProgress
  size?: number
}>()

const sz = computed(() => props.size ?? 32)
const radius = computed(() => (sz.value - 4) / 2)
const circumference = computed(() => 2 * Math.PI * radius.value)
const dashOffset = computed(() => {
  const pct = props.progress?.progress ?? 0
  return circumference.value * (1 - pct / 100)
})

const isProcessing = computed(() =>
  props.status === 'PENDING' && props.progress && props.progress.progress > 0
)

const stepLabel = computed(() => {
  if (!props.progress) return ''
  const labels: Record<string, string> = {
    received: 'Starting...',
    extracting_text: 'Extracting text',
    preprocessing_image: 'Preprocessing',
    running_ocr: 'Running OCR',
    extracting_metadata: 'Reading metadata',
    generating_thumbnail: 'Generating preview',
  }
  return labels[props.progress.step] ?? props.progress.step
})
</script>

<template>
  <div class="indicator" :class="`indicator--${status.toLowerCase()}`">
    <!-- PENDING without progress -->
    <div v-if="status === 'PENDING' && !isProcessing" class="indicator__icon indicator__icon--pending">
      <Clock :size="sz * 0.5" :stroke-width="2" />
    </div>

    <!-- PROCESSING (circular progress) -->
    <div v-else-if="isProcessing" class="indicator__circular" :title="stepLabel">
      <svg :width="sz" :height="sz" class="indicator__ring">
        <circle
          :cx="sz / 2" :cy="sz / 2" :r="radius"
          fill="none"
          stroke="var(--color-border-subtle)"
          :stroke-width="2.5"
        />
        <circle
          :cx="sz / 2" :cy="sz / 2" :r="radius"
          fill="none"
          stroke="var(--color-accent)"
          :stroke-width="2.5"
          stroke-linecap="round"
          :stroke-dasharray="circumference"
          :stroke-dashoffset="dashOffset"
          class="indicator__progress-ring"
        />
      </svg>
      <span class="indicator__pct">{{ progress!.progress }}%</span>
    </div>

    <!-- READY -->
    <div v-else-if="status === 'READY'" class="indicator__icon indicator__icon--ready">
      <Check :size="sz * 0.5" :stroke-width="2.5" />
    </div>

    <!-- FAILED -->
    <div v-else-if="status === 'FAILED'" class="indicator__icon indicator__icon--failed">
      <AlertCircle :size="sz * 0.5" :stroke-width="2" />
    </div>
  </div>
</template>

<style scoped>
.indicator {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.indicator__icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.indicator__icon--pending {
  color: var(--color-status-pending);
  animation: pulse 2s ease-in-out infinite;
}

.indicator__icon--ready {
  color: var(--color-status-ready);
  animation: pop-in 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.indicator__icon--failed {
  color: var(--color-status-failed);
  animation: shake 0.5s ease-in-out;
}

.indicator__circular {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.indicator__ring {
  transform: rotate(-90deg);
}

.indicator__progress-ring {
  transition: stroke-dashoffset 0.6s ease;
}

.indicator__pct {
  position: absolute;
  font-size: 8px;
  font-weight: 700;
  color: var(--color-accent);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

@keyframes pop-in {
  0% { transform: scale(0); opacity: 0; }
  100% { transform: scale(1); opacity: 1; }
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  20%, 60% { transform: translateX(-3px); }
  40%, 80% { transform: translateX(3px); }
}
</style>
