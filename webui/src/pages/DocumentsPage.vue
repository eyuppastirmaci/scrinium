<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { File as FileIcon, Upload } from '@lucide/vue'
import ViewToggle from '../components/ViewToggle.vue'
import DocumentGrid from '../features/documents/DocumentGrid.vue'
import DocumentTable from '../features/documents/DocumentTable.vue'
import { fetchDocuments, type DocumentSummary } from '../api/documents'

const documents = ref<DocumentSummary[]>([])
const viewMode = ref<'grid' | 'list'>('grid')
const loading = ref(true)
const error = ref(false)
const page = ref(0)
const hasNext = ref(false)
const loadingMore = ref(false)

async function load(p = 0) {
  try {
    const data = await fetchDocuments(p)
    if (p === 0) {
      documents.value = data.items
    } else {
      documents.value = [...documents.value, ...data.items]
    }
    page.value = data.page
    hasNext.value = data.hasNext
  } catch {
    if (p === 0) error.value = true
  }
}

async function loadMore() {
  loadingMore.value = true
  await load(page.value + 1)
  loadingMore.value = false
}

onMounted(async () => {
  await load()
  loading.value = false
})
</script>

<template>
  <div class="docs-page">
    <div v-if="loading" class="docs-page__status">Loading...</div>
    <div v-else-if="error" class="docs-page__error">Failed to load documents.</div>

    <template v-else-if="documents.length">
      <div class="docs-page__toolbar">
        <h2 class="docs-page__title">Documents</h2>
        <ViewToggle v-model="viewMode" />
      </div>

      <DocumentGrid v-if="viewMode === 'grid'" :documents="documents" />
      <DocumentTable v-else :documents="documents" />

      <button
        v-if="hasNext"
        class="docs-page__load-more"
        :disabled="loadingMore"
        @click="loadMore"
      >
        {{ loadingMore ? 'Loading...' : 'Load more' }}
      </button>
    </template>

    <div v-else class="empty-state">
      <FileIcon :size="40" :stroke-width="1" class="empty-state__icon" />
      <p class="empty-state__text">No documents yet</p>
      <RouterLink to="/upload" class="empty-state__link">
        <Upload :size="14" :stroke-width="2" />
        Upload files
      </RouterLink>
    </div>
  </div>
</template>

<style scoped>
.docs-page {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.docs-page__status {
  text-align: center;
  padding: 48px 0;
  color: var(--color-text-secondary);
  font-size: 14px;
}

.docs-page__error {
  background-color: var(--color-danger-subtle);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  color: var(--color-danger);
  font-size: 13px;
}

.docs-page__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.docs-page__title {
  font-size: 18px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.docs-page__load-more {
  margin: 20px auto 0;
  padding: 10px 24px;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  transition: color 0.15s, border-color 0.15s, background-color 0.15s;
}

.docs-page__load-more:hover:not(:disabled) {
  color: var(--color-text-primary);
  border-color: var(--color-accent);
  background-color: var(--color-accent-subtle);
}

.docs-page__load-more:disabled {
  opacity: 0.5;
  cursor: default;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 48px 0;
}

.empty-state__icon {
  color: var(--color-text-tertiary);
  margin-bottom: 12px;
}

.empty-state__text {
  font-size: 14px;
  color: var(--color-text-secondary);
  margin-bottom: 16px;
}

.empty-state__link {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  color: #fff;
  background-color: var(--color-accent);
  border-radius: var(--radius-md);
  transition: background-color 0.15s;
}

.empty-state__link:hover {
  background-color: var(--color-accent-hover);
}
</style>
