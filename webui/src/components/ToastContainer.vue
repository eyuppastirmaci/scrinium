<script setup lang="ts">
import { X } from '@lucide/vue'
import { useToast } from '../composables/useToast'

const { toasts, dismiss } = useToast()
</script>

<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="toast"
          :class="`toast--${toast.type}`"
        >
          <span class="toast__message">{{ toast.message }}</span>
          <button class="toast__close" @click="dismiss(toast.id)">
            <X :size="14" :stroke-width="2" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 9999;
  display: flex;
  flex-direction: column-reverse;
  gap: 8px;
}

.toast {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: var(--radius-md);
  font-size: 13px;
  color: var(--color-text-primary);
  background-color: var(--color-bg-elevated);
  border: 1px solid var(--color-border-default);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  min-width: 260px;
  max-width: 380px;
}

.toast--success {
  border-color: rgba(34, 197, 94, 0.4);
}

.toast--error {
  border-color: rgba(239, 68, 68, 0.4);
}

.toast__message {
  flex: 1;
}

.toast__close {
  color: var(--color-text-tertiary);
  flex-shrink: 0;
  transition: color 0.15s;
}

.toast__close:hover {
  color: var(--color-text-primary);
}

.toast-enter-active {
  transition: all 0.3s ease;
}

.toast-leave-active {
  transition: all 0.2s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(30px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
