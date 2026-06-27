import { get, del, ApiError } from './client'

export { ApiError }

export type DocumentStatus = 'PENDING' | 'READY' | 'FAILED' | 'DELETED'

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

export interface DocumentSummary {
  id: string
  fileName: string
  contentType: string
  sizeBytes: number
  status: DocumentStatus
  createdAt: string
}

export interface DocumentListResponse {
  items: DocumentSummary[]
  page: number
  size: number
  totalElements: number
  hasNext: boolean
}

export interface DocumentDetail {
  id: string
  fileName: string
  contentType: string
  sizeBytes: number
  sha256: string
  status: DocumentStatus
  failureReason: string | null
  createdAt: string
  updatedAt: string
}

export interface DocumentMetadata {
  documentId: string
  pageCount: number | null
  pdfCreatedAt: string | null
  pdfModifiedAt: string | null
  pdfAuthor: string | null
  pdfTitle: string | null
  imageCapturedAt: string | null
  imageDevice: string | null
  imageGpsPresent: boolean
  imageGpsRedacted: boolean
  detectedLanguage: string | null
  metadata: Record<string, unknown>
  createdAt: string
  updatedAt: string
}

export interface ExtractedPage {
  pageNumber: number
  text: string
}

export interface DocumentExtractedText {
  documentId: string
  pages: ExtractedPage[]
  combinedText: string
}

export function fetchUploadConstraints(): Promise<UploadConstraints> {
  return get<UploadConstraints>('/documents/upload-constraints')
}

export function fetchDocuments(page = 0, size = 20): Promise<DocumentListResponse> {
  return get<DocumentListResponse>(`/documents?page=${page}&size=${size}`)
}

export function fetchDocument(id: string): Promise<DocumentDetail> {
  return get<DocumentDetail>(`/documents/${id}`)
}

export function deleteDocument(id: string): Promise<void> {
  return del(`/documents/${id}`)
}

export function fetchDocumentMetadata(id: string): Promise<DocumentMetadata> {
  return get<DocumentMetadata>(`/documents/${id}/metadata`)
}

export function fetchDocumentText(id: string): Promise<DocumentExtractedText> {
  return get<DocumentExtractedText>(`/documents/${id}/text`)
}

export function getDownloadUrl(id: string): string {
  return `/api/documents/${id}/download`
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
      } else if (xhr.status === 409) {
        try {
          const parsed = JSON.parse(xhr.responseText)
          const err = new ApiError(409, parsed.detail ?? 'Duplicate document', {
            existingDocumentId: parsed.existingDocumentId,
          })
          reject(err)
        } catch {
          reject(new ApiError(409, 'Duplicate document'))
        }
      } else {
        let message: string
        try {
          const parsed = JSON.parse(xhr.responseText)
          message = parsed.message ?? parsed.error ?? xhr.statusText
        } catch {
          message = xhr.responseText || xhr.statusText
        }
        reject(new ApiError(xhr.status, message))
      }
    })

    xhr.addEventListener('error', () => reject(new ApiError(0, 'Network error')))
    xhr.addEventListener('abort', () => reject(new ApiError(0, 'Upload cancelled')))

    xhr.send(formData)
  })
}
