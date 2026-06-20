import { get } from './client'

export interface UploadConstraints {
  supportedContentTypes: string[]
  maxFileSize: number
  maxFileSizeLabel: string
  maxFilesPerRequest: number
}

export interface UploadResult {
  id: string | null
  fileName: string | null
  status: string
  error: string | null
}

export function fetchUploadConstraints(): Promise<UploadConstraints> {
  return get<UploadConstraints>('/documents/upload-constraints')
}

export function uploadSingleFile(
  file: File,
  onProgress: (percent: number) => void
): Promise<UploadResult> {
  return new Promise((resolve, reject) => {
    const formData = new FormData()
    formData.append('file', file)

    const xhr = new XMLHttpRequest()
    xhr.open('POST', '/api/documents')

    xhr.upload.addEventListener('progress', (e) => {
      if (e.lengthComputable) {
        onProgress(Math.round((e.loaded / e.total) * 100))
      }
    })

    xhr.addEventListener('load', () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        const results: UploadResult[] = JSON.parse(xhr.responseText)
        resolve(results[0])
      } else {
        let message: string
        try {
          const parsed = JSON.parse(xhr.responseText)
          message = parsed.message ?? parsed.error ?? xhr.statusText
        } catch {
          message = xhr.responseText || xhr.statusText
        }
        reject(new Error(message))
      }
    })

    xhr.addEventListener('error', () => reject(new Error('Network error')))
    xhr.addEventListener('abort', () => reject(new Error('Upload cancelled')))

    xhr.send(formData)
  })
}
