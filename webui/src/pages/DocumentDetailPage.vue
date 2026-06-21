<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, Download, Trash2 } from '@lucide/vue'
import StatusBadge from '../components/StatusBadge.vue'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import { fetchDocument, deleteDocument, getDownloadUrl, type DocumentDetail } from '../api/documents'
import { formatFileSize, formatDate } from '../utils/format'

const props = defineProps<{ id: string }>()
const router = useRouter()

const document = ref<DocumentDetail | null>(null)
const loading = ref(true)
const error = ref(false)
const showDeleteConfirm = ref(false)
const deleting = ref(false)

onMounted(async () => {
  try {
    document.value = await fetchDocument(props.id)
  } catch {
    error.value = true
  } finally {
    loading.value = false
  }
})

async function onDelete() {
  deleting.value = true
  try {
    await deleteDocument(props.id)
    router.push('/')
  } catch {
    deleting.value = false
    showDeleteConfirm.value = false
  }
}
</script>

<template>
  <div class="detail">
    <RouterLink to="/" class="detail__back">
      <ArrowLeft :size="16" :stroke-width="1.5" />
      Back to documents
    </RouterLink>

    <div v-if="loading" class="detail__status">Loading...</div>
    <div v-else-if="error" class="detail__error">Document not found.</div>

    <template v-else-if="document">
      <div class="detail__card">
        <div class="detail__header">
          <div>
            <h1 class="detail__title">{{ document.fileName }}</h1>
            <p class="detail__subtitle">Uploaded {{ formatDate(document.createdAt) }}</p>
          </div>
          <StatusBadge :status="document.status" />
        </div>

        <hr class="detail__divider" />

        <div class="detail__grid">
          <div class="detail__field">
            <span class="detail__label">File type</span>
            <span class="detail__value">{{ document.contentType }}</span>
          </div>
          <div class="detail__field">
            <span class="detail__label">Size</span>
            <span class="detail__value">{{ formatFileSize(document.sizeBytes) }}</span>
          </div>
          <div class="detail__field detail__field--full">
            <span class="detail__label">SHA-256</span>
            <span class="detail__value detail__value--mono">{{ document.sha256 }}</span>
          </div>
          <div class="detail__field">
            <span class="detail__label">Status</span>
            <span class="detail__value">{{ document.status }}</span>
          </div>
          <div class="detail__field">
            <span class="detail__label">Created</span>
            <span class="detail__value">{{ formatDate(document.createdAt) }}</span>
          </div>
          <div class="detail__field">
            <span class="detail__label">Last updated</span>
            <span class="detail__value">{{ formatDate(document.updatedAt) }}</span>
          </div>
        </div>

        <hr class="detail__divider" />

        <div class="detail__actions">
          <a
            :href="getDownloadUrl(document.id)"
            download
            class="detail__btn detail__btn--primary"
            :class="{ 'detail__btn--disabled': document.status !== 'READY' }"
          >
            <Download :size="15" :stroke-width="2" />
            Download
          </a>
          <button class="detail__btn detail__btn--danger" @click="showDeleteConfirm = true">
            <Trash2 :size="15" :stroke-width="2" />
            Delete
          </button>
        </div>
      </div>

      <ConfirmDialog
        :open="showDeleteConfirm"
        title="Delete document"
        :message="`Are you sure you want to delete '${document.fileName}'? This action cannot be undone.`"
        @confirm="onDelete"
        @cancel="showDeleteConfirm = false"
      />
    </template>
  </div>
</template>

<style scoped>
.detail__back {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-bottom: 20px;
  transition: color 0.15s;
}

.detail__back:hover {
  color: var(--color-accent);
}

.detail__status {
  text-align: center;
  padding: 48px 0;
  color: var(--color-text-secondary);
  font-size: 14px;
}

.detail__error {
  background-color: var(--color-danger-subtle);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  color: var(--color-danger);
  font-size: 13px;
}

.detail__card {
  background-color: var(--color-bg-surface);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  padding: 24px;
}

.detail__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.detail__title {
  font-size: 18px;
  font-weight: 500;
  color: var(--color-text-primary);
  word-break: break-all;
}

.detail__subtitle {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-top: 4px;
}

.detail__divider {
  border: none;
  border-top: 1px solid var(--color-border-subtle);
  margin: 20px 0;
}

.detail__grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.detail__field--full {
  grid-column: 1 / -1;
}

.detail__label {
  display: block;
  font-size: 11px;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.3px;
  margin-bottom: 2px;
}

.detail__value {
  font-size: 14px;
  color: var(--color-text-primary);
}

.detail__value--mono {
  font-family: var(--font-mono);
  font-size: 12px;
  word-break: break-all;
  color: var(--color-text-secondary);
}

.detail__actions {
  display: flex;
  gap: 10px;
}

.detail__btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 9px 18px;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  transition: background-color 0.15s, opacity 0.15s;
}

.detail__btn--primary {
  color: #fff;
  background-color: var(--color-accent);
}

.detail__btn--primary:hover {
  background-color: var(--color-accent-hover);
}

.detail__btn--disabled {
  opacity: 0.4;
  pointer-events: none;
}

.detail__btn--danger {
  color: var(--color-danger);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.detail__btn--danger:hover {
  background-color: var(--color-danger-subtle);
}
</style>
