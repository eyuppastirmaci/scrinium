<script setup lang="ts">
import { LayoutGrid, List } from '@lucide/vue'

defineProps<{ modelValue: 'grid' | 'list' }>()
const emit = defineEmits<{ 'update:modelValue': [value: 'grid' | 'list'] }>()
</script>

<template>
  <div class="toggle">
    <button
      class="toggle__btn"
      :class="{ 'toggle__btn--active': modelValue === 'grid' }"
      @click="emit('update:modelValue', 'grid')"
    >
      <LayoutGrid :size="16" :stroke-width="1.5" />
    </button>
    <button
      class="toggle__btn"
      :class="{ 'toggle__btn--active': modelValue === 'list' }"
      @click="emit('update:modelValue', 'list')"
    >
      <List :size="16" :stroke-width="1.5" />
    </button>
  </div>
</template>

<style scoped>
.toggle {
  display: flex;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.toggle__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 30px;
  color: var(--color-text-secondary);
  transition: color 0.15s, background-color 0.15s;
}

.toggle__btn + .toggle__btn {
  border-left: 1px solid var(--color-border-default);
}

.toggle__btn:hover {
  color: var(--color-text-primary);
  background-color: var(--color-bg-elevated);
}

.toggle__btn--active {
  color: var(--color-accent);
  background-color: var(--color-accent-subtle);
}
</style>
