<script setup lang="ts">
import type { DocumentStatus } from '../api/documents'

defineProps<{ status: DocumentStatus }>()

const labels: Record<DocumentStatus, string> = {
  PENDING: 'Processing',
  READY: 'Ready',
  FAILED: 'Failed',
  DELETED: 'Deleted',
}
</script>

<template>
  <span class="badge" :class="`badge--${status.toLowerCase()}`">
    {{ labels[status] }}
  </span>
</template>

<style scoped>
.badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 10px;
  border-radius: var(--radius-pill);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.badge--ready {
  background-color: var(--color-status-ready-bg);
  color: var(--color-status-ready);
}

.badge--pending {
  background-color: var(--color-status-pending-bg);
  color: var(--color-status-pending);
}

.badge--failed {
  background-color: var(--color-status-failed-bg);
  color: var(--color-status-failed);
}

.badge--deleted {
  background-color: var(--color-danger-subtle);
  color: var(--color-text-secondary);
}
</style>
