<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { File as FileIcon, RotateCcw } from '@lucide/vue'
import DropZone from '../features/upload/DropZone.vue'
import FileList from '../features/upload/FileList.vue'
import { fetchUploadConstraints, uploadSingleFile, type UploadConstraints } from '../api/documents'
import type { StagedFile } from '../features/upload/types'
import { formatFileSize } from '../utils/format'

const constraints = ref<UploadConstraints | null>(null)
const loading = ref(true)
const loadError = ref(false)
const stagedFiles = ref<StagedFile[]>([])

const isUploading = computed(() =>
  stagedFiles.value.some((f) => f.status === 'uploading')
)

const allDone = computed(() =>
  stagedFiles.value.length > 0 &&
  stagedFiles.value.every((f) => f.status === 'done' || f.status === 'error' || f.status === 'rejected')
)

onMounted(async () => {
  try {
    constraints.value = await fetchUploadConstraints()
  } catch {
    loadError.value = true
  } finally {
    loading.value = false
  }
})

function validateFile(file: File): string | null {
  if (!constraints.value) return null

  if (!constraints.value.supportedContentTypes.includes(file.type)) {
    const ext = file.name.split('.').pop()?.toUpperCase() ?? file.type
    return `Unsupported file type: ${ext}`
  }

  if (file.size > constraints.value.maxFileSize) {
    return `File too large (${formatFileSize(file.size)}). Max ${constraints.value.maxFileSizeLabel}.`
  }

  if (file.size === 0) {
    return 'File is empty'
  }

  return null
}

function onFiles(files: File[]) {
  const newEntries: StagedFile[] = files.map((file) => {
    const rejectReason = validateFile(file)
    if (rejectReason) {
      return { file, status: 'rejected' as const, progress: 0, rejectReason }
    }
    return { file, status: 'idle' as const, progress: 0 }
  })
  stagedFiles.value = [...stagedFiles.value, ...newEntries]
}

function removeFile(index: number) {
  stagedFiles.value = stagedFiles.value.filter((_, i) => i !== index)
}

function clearFiles() {
  stagedFiles.value = []
}

async function doUpload() {
  const uploadable = stagedFiles.value.filter((f) => f.status === 'idle')

  const uploads = uploadable.map(async (staged) => {
    staged.status = 'uploading'
    staged.progress = 0

    try {
      await uploadSingleFile(staged.file, (percent) => {
        staged.progress = percent
      })
      staged.progress = 100
      staged.status = 'done'
    } catch (e: unknown) {
      staged.status = 'error'
      staged.error = (e as Error).message || 'Upload failed'
    }
  })

  await Promise.all(uploads)
}

function reset() {
  stagedFiles.value = []
}
</script>

<template>
  <div class="upload-page" :class="{ 'upload-page--has-files': stagedFiles.length > 0 }">
    <div v-if="loading" class="upload-page__status">Loading...</div>
    <div v-else-if="loadError" class="upload-page__error">Failed to connect to the server.</div>

    <template v-else-if="constraints">
      <DropZone
        v-if="!isUploading && !allDone"
        :max-file-size-label="constraints.maxFileSizeLabel"
        @files="onFiles"
      />

      <FileList
        v-if="stagedFiles.length"
        :files="stagedFiles"
        @remove="removeFile"
        @clear="clearFiles"
        @upload="doUpload"
      />

      <button v-if="allDone" class="upload-page__reset" @click="reset">
        <RotateCcw :size="15" :stroke-width="2" />
        Upload again
      </button>

      <div v-if="!stagedFiles.length" class="empty-state">
        <FileIcon :size="40" :stroke-width="1" class="empty-state__icon" />
        <p class="empty-state__text">No documents yet</p>
      </div>
    </template>
  </div>
</template>

<style scoped>
.upload-page {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.upload-page--has-files {
  flex: 1;
}

.upload-page__status {
  text-align: center;
  padding: 48px 0;
  color: var(--color-text-secondary);
  font-size: 14px;
}

.upload-page__error {
  background-color: var(--color-danger-subtle);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: var(--radius-md);
  padding: 12px 16px;
  color: var(--color-danger);
  font-size: 13px;
}

.upload-page__reset {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 16px auto 0;
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 500;
  color: #fff;
  background-color: var(--color-accent);
  border-radius: var(--radius-md);
  transition: background-color 0.15s;
}

.upload-page__reset:hover {
  background-color: var(--color-accent-hover);
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
}
</style>
