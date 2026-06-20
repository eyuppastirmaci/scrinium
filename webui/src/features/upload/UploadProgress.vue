<script setup lang="ts">
import { computed } from 'vue'
import { Check, X } from '@lucide/vue'

const props = withDefaults(defineProps<{
  percent: number
  status: 'uploading' | 'done' | 'error'
  size?: number
}>(), {
  size: 36,
})

const strokeWidth = computed(() => props.size >= 80 ? 6 : 3)
const ringRadius = computed(() => (props.size / 2) - strokeWidth.value - 2)
const circumference = computed(() => 2 * Math.PI * ringRadius.value)
const dashOffset = computed(() =>
  circumference.value - (circumference.value * props.percent) / 100
)
const iconSize = computed(() => Math.round(props.size * 0.45))
const fontSize = computed(() => Math.round(props.size * 0.24) + 'px')
const innerRadius = computed(() => (props.size / 2) - 2)
</script>

<template>
  <div class="progress-ring" :style="{ width: size + 'px', height: size + 'px' }">
    <!-- Done state: solid green circle + white check -->
    <template v-if="status === 'done'">
      <svg :width="size" :height="size" :viewBox="`0 0 ${size} ${size}`">
        <circle
          :cx="size / 2" :cy="size / 2" :r="innerRadius"
          fill="var(--color-status-ready)"
        />
      </svg>
      <div class="progress-ring__content">
        <Check :size="iconSize" :stroke-width="3" class="progress-ring__icon--white" />
      </div>
    </template>

    <!-- Error state: solid red circle + white X -->
    <template v-else-if="status === 'error'">
      <svg :width="size" :height="size" :viewBox="`0 0 ${size} ${size}`">
        <circle
          :cx="size / 2" :cy="size / 2" :r="innerRadius"
          fill="var(--color-danger)"
        />
      </svg>
      <div class="progress-ring__content">
        <X :size="iconSize" :stroke-width="3" class="progress-ring__icon--white" />
      </div>
    </template>

    <!-- Uploading state: track ring + progress arc + percentage -->
    <template v-else>
      <svg class="progress-ring__svg--rotate" :width="size" :height="size" :viewBox="`0 0 ${size} ${size}`">
        <circle
          :cx="size / 2" :cy="size / 2" :r="ringRadius"
          fill="none"
          stroke="var(--color-border-default)"
          :stroke-width="strokeWidth"
        />
        <circle
          class="progress-ring__arc"
          :cx="size / 2" :cy="size / 2" :r="ringRadius"
          fill="none"
          stroke="var(--color-accent)"
          :stroke-width="strokeWidth + 1"
          stroke-linecap="round"
          :stroke-dasharray="circumference"
          :stroke-dashoffset="dashOffset"
        />
      </svg>
      <div class="progress-ring__content">
        <span class="progress-ring__percent" :style="{ fontSize }">{{ percent }}%</span>
      </div>
    </template>
  </div>
</template>

<style scoped>
.progress-ring {
  position: relative;
  flex-shrink: 0;
}

.progress-ring svg.progress-ring__svg--rotate {
  transform: rotate(-90deg);
}

.progress-ring__arc {
  transition: stroke-dashoffset 0.3s ease;
}

.progress-ring__content {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.progress-ring__percent {
  font-weight: 900;
  font-family: var(--font-mono);
  color: #fff;
}

.progress-ring__icon--white {
  color: #fff;
}
</style>
