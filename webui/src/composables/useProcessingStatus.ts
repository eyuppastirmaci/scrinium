import { reactive, onUnmounted } from 'vue'
import { useToast } from './useToast'

export interface ProcessingProgress {
  step: string
  progress: number
}

const state = reactive<Record<string, ProcessingProgress>>({})
const completedIds = reactive(new Set<string>())
const failedIds = reactive(new Set<string>())

let eventSource: EventSource | null = null
let refCount = 0
let toastInitialized = false

function connect() {
  if (eventSource) return

  const { show } = useToast()

  eventSource = new EventSource('/api/documents/status/stream')

  eventSource.addEventListener('progress', (e) => {
    const data = JSON.parse((e as MessageEvent).data)
    const id: string = data.documentId
    const progress: number = data.progress
    const step: string = data.step

    if (step === 'completed') {
      delete state[id]
      completedIds.add(id)
      if (toastInitialized) {
        show('Document processing completed', 'success')
      }
    } else if (step === 'failed') {
      delete state[id]
      failedIds.add(id)
      if (toastInitialized) {
        show('Document processing failed', 'error')
      }
    } else {
      state[id] = { step, progress }
    }
  })

  eventSource.onerror = () => {
    eventSource?.close()
    eventSource = null
    setTimeout(connect, 3000)
  }

  // Suppress toasts for events received on initial connection.
  setTimeout(() => { toastInitialized = true }, 2000)
}

function disconnect() {
  if (eventSource) {
    eventSource.close()
    eventSource = null
  }
  toastInitialized = false
}

export function useProcessingStatus() {
  refCount++
  if (refCount === 1) connect()

  onUnmounted(() => {
    refCount--
    if (refCount === 0) disconnect()
  })

  return { processingStatus: state, completedIds, failedIds }
}
