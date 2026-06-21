<script setup lang="ts">
import { watch } from 'vue'

const props = withDefaults(defineProps<{
  open: boolean
  title: string
  message: string
  confirmLabel?: string
}>(), {
  confirmLabel: 'Delete',
})

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('cancel')
}

watch(() => props.open, (val) => {
  if (val) {
    document.addEventListener('keydown', onKeydown)
  } else {
    document.removeEventListener('keydown', onKeydown)
  }
})
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="overlay" @click.self="emit('cancel')">
      <div class="dialog">
        <h3 class="dialog__title">{{ title }}</h3>
        <p class="dialog__message">{{ message }}</p>
        <div class="dialog__actions">
          <button class="dialog__btn dialog__btn--cancel" @click="emit('cancel')">
            Cancel
          </button>
          <button class="dialog__btn dialog__btn--confirm" @click="emit('confirm')">
            {{ confirmLabel }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background-color: var(--color-bg-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  background-color: var(--color-bg-elevated);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  padding: 24px;
  width: 100%;
  max-width: 400px;
}

.dialog__title {
  font-size: 16px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: 8px;
}

.dialog__message {
  font-size: 14px;
  color: var(--color-text-secondary);
  margin-bottom: 24px;
  line-height: 1.5;
}

.dialog__actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.dialog__btn {
  padding: 8px 16px;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  transition: background-color 0.15s;
}

.dialog__btn--cancel {
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border-default);
}

.dialog__btn--cancel:hover {
  background-color: var(--color-bg-surface);
}

.dialog__btn--confirm {
  color: #fff;
  background-color: var(--color-danger);
}

.dialog__btn--confirm:hover {
  background-color: var(--color-danger-hover);
}
</style>
