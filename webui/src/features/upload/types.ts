export interface StagedFile {
  file: File
  status: 'idle' | 'rejected' | 'uploading' | 'done' | 'error'
  progress: number
  error?: string
  rejectReason?: string
  duplicateId?: string
}
