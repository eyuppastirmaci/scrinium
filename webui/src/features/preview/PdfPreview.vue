<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { ChevronLeft, ChevronRight } from '@lucide/vue'
import type { PDFDocumentProxy } from 'pdfjs-dist'
import * as pdfjsLib from 'pdfjs-dist'

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.min.mjs',
  import.meta.url,
).toString()

const props = defineProps<{ url: string }>()

const wrapperRef = ref<HTMLDivElement | null>(null)
const canvasRef = ref<HTMLCanvasElement | null>(null)
const loading = ref(true)
const error = ref(false)
const currentPage = ref(1)
const totalPages = ref(0)

let pdfDoc: PDFDocumentProxy | null = null
let loadingTask: ReturnType<typeof pdfjsLib.getDocument> | null = null

onMounted(() => loadPdf())

onUnmounted(() => {
  if (loadingTask) {
    loadingTask.destroy()
    loadingTask = null
  }
  pdfDoc = null
})

async function loadPdf() {
  loading.value = true
  error.value = false
  try {
    loadingTask = pdfjsLib.getDocument({ url: props.url })
    pdfDoc = await loadingTask.promise
    totalPages.value = pdfDoc.numPages
    loading.value = false
    await nextTick()
    await renderPage(1)
  } catch {
    error.value = true
    loading.value = false
  }
}

async function renderPage(num: number) {
  if (!pdfDoc || !canvasRef.value || !wrapperRef.value) return
  const page = await pdfDoc.getPage(num)
  const baseViewport = page.getViewport({ scale: 1 })
  const wrapperStyle = getComputedStyle(wrapperRef.value)
  const paddingX = parseFloat(wrapperStyle.paddingLeft) + parseFloat(wrapperStyle.paddingRight)
  const paddingY = parseFloat(wrapperStyle.paddingTop) + parseFloat(wrapperStyle.paddingBottom)
  const availableWidth = wrapperRef.value.clientWidth - paddingX
  const availableHeight = window.innerHeight - wrapperRef.value.getBoundingClientRect().top - paddingY - 24
  const fitScale = Math.min(availableWidth / baseViewport.width, availableHeight / baseViewport.height)
  const displayWidth = baseViewport.width * fitScale
  const displayHeight = baseViewport.height * fitScale
  const renderScale = fitScale * window.devicePixelRatio
  const viewport = page.getViewport({ scale: renderScale })
  const canvas = canvasRef.value
  canvas.width = viewport.width
  canvas.height = viewport.height
  canvas.style.width = `${displayWidth}px`
  canvas.style.height = `${displayHeight}px`
  await page.render({ canvas, viewport }).promise
}

async function prevPage() {
  if (currentPage.value <= 1) return
  currentPage.value--
  await renderPage(currentPage.value)
}

async function nextPage() {
  if (currentPage.value >= totalPages.value) return
  currentPage.value++
  await renderPage(currentPage.value)
}
</script>

<template>
  <div class="pdf-preview">
    <div v-if="loading" class="pdf-preview__status">Loading PDF...</div>

    <div v-else-if="error" class="pdf-preview__error">
      Failed to load PDF preview.
    </div>

    <template v-else>
      <div v-if="totalPages > 1" class="pdf-preview__nav">
        <button
          class="pdf-preview__nav-btn"
          :disabled="currentPage <= 1"
          @click="prevPage"
        >
          <ChevronLeft :size="16" :stroke-width="2" />
        </button>
        <span class="pdf-preview__page-info">
          Page {{ currentPage }} of {{ totalPages }}
        </span>
        <button
          class="pdf-preview__nav-btn"
          :disabled="currentPage >= totalPages"
          @click="nextPage"
        >
          <ChevronRight :size="16" :stroke-width="2" />
        </button>
      </div>
      <div ref="wrapperRef" class="pdf-preview__canvas-wrapper">
        <canvas ref="canvasRef" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.pdf-preview {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.pdf-preview__status {
  text-align: center;
  padding: 48px 0;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.pdf-preview__error {
  text-align: center;
  padding: 48px 0;
  color: var(--color-danger);
  font-size: 13px;
}

.pdf-preview__nav {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.pdf-preview__nav-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border-default);
  transition: color 0.15s, background-color 0.15s;
}

.pdf-preview__nav-btn:hover:not(:disabled) {
  color: var(--color-text-primary);
  background-color: var(--color-bg-elevated);
}

.pdf-preview__nav-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.pdf-preview__page-info {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
  min-width: 100px;
  text-align: center;
}

.pdf-preview__canvas-wrapper {
  display: flex;
  justify-content: center;
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  background-color: var(--color-bg-base);
  padding: 8px;
}

.pdf-preview__canvas-wrapper canvas {
  display: block;
}
</style>
