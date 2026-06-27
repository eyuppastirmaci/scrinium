<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, Download, Trash2, Copy, Check, FileDown, RotateCcw } from '@lucide/vue'
import StatusBadge from '../components/StatusBadge.vue'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import {
  fetchDocument,
  deleteDocument,
  retryProcessing,
  getDownloadUrl,
  fetchDocumentMetadata,
  fetchDocumentText,
  type DocumentDetail,
  type DocumentMetadata,
  type DocumentExtractedText,
} from '../api/documents'
import { formatFileSize, formatDate } from '../utils/format'
import DocumentPreview from '../features/preview/DocumentPreview.vue'

const props = defineProps<{ id: string }>()
const router = useRouter()

type Tab = 'overview' | 'metadata' | 'text' | 'preview'
const activeTab = ref<Tab>('overview')

const document = ref<DocumentDetail | null>(null)
const loading = ref(true)
const error = ref(false)
const showDeleteConfirm = ref(false)
const deleting = ref(false)

const metadata = ref<DocumentMetadata | null>(null)
const metadataLoading = ref(false)
const metadataError = ref(false)
const metadataEmpty = ref(false)

const extractedText = ref<DocumentExtractedText | null>(null)
const textLoading = ref(false)
const textError = ref(false)
const textEmpty = ref(false)

onMounted(async () => {
  try {
    document.value = await fetchDocument(props.id)
  } catch {
    error.value = true
  } finally {
    loading.value = false
  }
})

watch(activeTab, (tab) => {
  if (tab === 'metadata' && !metadata.value && !metadataLoading.value && !metadataEmpty.value) {
    loadMetadata()
  }
  if (tab === 'text' && !extractedText.value && !textLoading.value && !textEmpty.value) {
    loadText()
  }
})

async function loadMetadata() {
  metadataLoading.value = true
  metadataError.value = false
  metadataEmpty.value = false
  try {
    metadata.value = await fetchDocumentMetadata(props.id)
  } catch (err: any) {
    if (err?.status === 404) {
      metadataEmpty.value = true
    } else {
      metadataError.value = true
    }
  } finally {
    metadataLoading.value = false
  }
}

async function loadText() {
  textLoading.value = true
  textError.value = false
  textEmpty.value = false
  try {
    const result = await fetchDocumentText(props.id)
    if (result.pages.length === 0) {
      textEmpty.value = true
    } else {
      extractedText.value = result
    }
  } catch (err: any) {
    if (err?.status === 404) {
      textEmpty.value = true
    } else {
      textError.value = true
    }
  } finally {
    textLoading.value = false
  }
}

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

const retrying = ref(false)

async function onRetry() {
  retrying.value = true
  try {
    await retryProcessing(props.id)
    document.value = await fetchDocument(props.id)
  } catch {
    // retry failed
  } finally {
    retrying.value = false
  }
}

const copied = ref(false)
let copiedTimeout: ReturnType<typeof setTimeout> | null = null

async function copyText() {
  if (!extractedText.value) return
  try {
    await navigator.clipboard.writeText(extractedText.value.combinedText)
    copied.value = true
    if (copiedTimeout) clearTimeout(copiedTimeout)
    copiedTimeout = setTimeout(() => { copied.value = false }, 2000)
  } catch {
    // clipboard API not available
  }
}

function downloadText() {
  if (!extractedText.value || !document.value) return
  const blob = new Blob([extractedText.value.combinedText], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = window.document.createElement('a')
  a.href = url
  const baseName = document.value.fileName.replace(/\.[^.]+$/, '')
  a.download = `${baseName}.extracted.txt`
  a.click()
  URL.revokeObjectURL(url)
}

function isPdf(doc: DocumentDetail): boolean {
  return doc.contentType === 'application/pdf'
}

function isImage(doc: DocumentDetail): boolean {
  return doc.contentType.startsWith('image/')
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

        <nav class="detail__tabs">
          <button
            class="detail__tab"
            :class="{ 'detail__tab--active': activeTab === 'overview' }"
            @click="activeTab = 'overview'"
          >
            Overview
          </button>
          <button
            class="detail__tab"
            :class="{ 'detail__tab--active': activeTab === 'metadata' }"
            @click="activeTab = 'metadata'"
          >
            Metadata
          </button>
          <button
            class="detail__tab"
            :class="{ 'detail__tab--active': activeTab === 'text' }"
            @click="activeTab = 'text'"
          >
            Extracted Text
          </button>
          <button
            class="detail__tab"
            :class="{ 'detail__tab--active': activeTab === 'preview' }"
            @click="activeTab = 'preview'"
          >
            Preview
          </button>
        </nav>

        <!-- Overview tab -->
        <div v-if="activeTab === 'overview'">
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
            <div v-if="document.status === 'FAILED' && document.failureReason" class="detail__field detail__field--full">
              <span class="detail__label">Processing error</span>
              <div class="detail__failure-reason">{{ document.failureReason }}</div>
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
            <button
              v-if="document.status === 'FAILED'"
              class="detail__btn detail__btn--secondary"
              :disabled="retrying"
              @click="onRetry"
            >
              <RotateCcw :size="15" :stroke-width="2" />
              {{ retrying ? 'Retrying...' : 'Retry' }}
            </button>
            <button class="detail__btn detail__btn--danger" @click="showDeleteConfirm = true">
              <Trash2 :size="15" :stroke-width="2" />
              Delete
            </button>
          </div>
        </div>

        <!-- Metadata tab -->
        <div v-else-if="activeTab === 'metadata'">
          <div v-if="metadataLoading" class="detail__tab-status">Loading metadata...</div>

          <div v-else-if="metadataError" class="detail__tab-error">
            Failed to load metadata.
            <button class="detail__retry" @click="loadMetadata">Retry</button>
          </div>

          <div v-else-if="metadataEmpty" class="detail__tab-empty">
            No metadata available for this document.
          </div>

          <div v-else-if="metadata" class="detail__grid">
            <!-- Common -->
            <div v-if="metadata.pageCount != null" class="detail__field">
              <span class="detail__label">Page count</span>
              <span class="detail__value">{{ metadata.pageCount }}</span>
            </div>
            <div v-if="metadata.detectedLanguage" class="detail__field">
              <span class="detail__label">Detected language</span>
              <span class="detail__value">{{ metadata.detectedLanguage }}</span>
            </div>

            <!-- PDF-specific -->
            <template v-if="isPdf(document)">
              <div v-if="metadata.pdfTitle" class="detail__field">
                <span class="detail__label">PDF title</span>
                <span class="detail__value">{{ metadata.pdfTitle }}</span>
              </div>
              <div v-if="metadata.pdfAuthor" class="detail__field">
                <span class="detail__label">Author</span>
                <span class="detail__value">{{ metadata.pdfAuthor }}</span>
              </div>
              <div v-if="metadata.pdfCreatedAt" class="detail__field">
                <span class="detail__label">PDF created</span>
                <span class="detail__value">{{ formatDate(metadata.pdfCreatedAt) }}</span>
              </div>
              <div v-if="metadata.pdfModifiedAt" class="detail__field">
                <span class="detail__label">PDF modified</span>
                <span class="detail__value">{{ formatDate(metadata.pdfModifiedAt) }}</span>
              </div>
            </template>

            <!-- Image-specific -->
            <template v-if="isImage(document)">
              <div v-if="metadata.imageCapturedAt" class="detail__field">
                <span class="detail__label">Capture date</span>
                <span class="detail__value">{{ formatDate(metadata.imageCapturedAt) }}</span>
              </div>
              <div v-if="metadata.imageDevice" class="detail__field">
                <span class="detail__label">Capture device</span>
                <span class="detail__value">{{ metadata.imageDevice }}</span>
              </div>
              <div class="detail__field">
                <span class="detail__label">GPS data</span>
                <span class="detail__value">
                  {{ metadata.imageGpsPresent
                    ? (metadata.imageGpsRedacted ? 'Present (redacted)' : 'Present')
                    : 'Not present' }}
                </span>
              </div>
            </template>
          </div>
        </div>

        <!-- Extracted Text tab -->
        <div v-else-if="activeTab === 'text'">
          <div v-if="textLoading" class="detail__tab-status">Loading extracted text...</div>

          <div v-else-if="textError" class="detail__tab-error">
            Failed to load extracted text.
            <button class="detail__retry" @click="loadText">Retry</button>
          </div>

          <div v-else-if="textEmpty" class="detail__tab-empty">
            No extracted text available for this document.
          </div>

          <div v-else-if="extractedText" class="text-view">
            <div class="text-view__actions">
              <button class="detail__btn detail__btn--secondary" @click="copyText">
                <component :is="copied ? Check : Copy" :size="15" :stroke-width="2" />
                {{ copied ? 'Copied' : 'Copy all' }}
              </button>
              <button class="detail__btn detail__btn--secondary" @click="downloadText">
                <FileDown :size="15" :stroke-width="2" />
                Download as .txt
              </button>
            </div>
            <div
              v-for="page in extractedText.pages"
              :key="page.pageNumber"
              class="text-view__page"
            >
              <div class="text-view__page-header">Page {{ page.pageNumber }}</div>
              <pre class="text-view__content">{{ page.text }}</pre>
            </div>
          </div>
        </div>

        <!-- Preview tab -->
        <div v-else-if="activeTab === 'preview'">
          <DocumentPreview
            v-if="document.status === 'READY'"
            :content-type="document.contentType"
            :preview-url="getDownloadUrl(document.id)"
          />
          <div v-else class="detail__tab-empty">
            Preview is not available while the document is being processed.
          </div>
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

.detail__tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--color-border-subtle);
  margin: 20px 0;
}

.detail__tab {
  padding: 10px 20px;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: color 0.15s, border-color 0.15s;
}

.detail__tab:hover {
  color: var(--color-text-primary);
}

.detail__tab--active {
  color: var(--color-accent);
  border-bottom-color: var(--color-accent);
}

.detail__tab-status {
  text-align: center;
  padding: 32px 0;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.detail__tab-error {
  text-align: center;
  padding: 32px 0;
  color: var(--color-danger);
  font-size: 13px;
}

.detail__retry {
  display: inline-block;
  margin-top: 8px;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-accent);
  transition: color 0.15s;
}

.detail__retry:hover {
  color: var(--color-accent-hover);
}

.detail__tab-empty {
  text-align: center;
  padding: 32px 0;
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.detail__failure-reason {
  font-size: 13px;
  color: var(--color-danger);
  background-color: var(--color-danger-subtle);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: var(--radius-sm);
  padding: 8px 12px;
  margin-top: 4px;
  word-break: break-word;
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

.detail__btn--secondary {
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border-default);
}

.detail__btn--secondary:hover {
  color: var(--color-text-primary);
  background-color: var(--color-bg-elevated);
}

.text-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.text-view__actions {
  display: flex;
  gap: 10px;
}

.text-view__page {
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.text-view__page-header {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  color: var(--color-text-tertiary);
  padding: 8px 16px;
  background-color: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border-subtle);
}

.text-view__content {
  padding: 16px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--color-text-primary);
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 400px;
  overflow-y: auto;
  margin: 0;
}
</style>
