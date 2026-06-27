import { get } from './client'

export interface SearchResultItem {
  documentId: string
  fileName: string
  snippet: string
  score: number
}

export interface SearchResponse {
  query: string | null
  items: SearchResultItem[]
  totalCount: number
  page: number
  size: number
}

export interface SearchFilters {
  type?: string
  dateFrom?: string
  dateTo?: string
  docDateFrom?: string
  docDateTo?: string
  minPages?: number
  maxPages?: number
}

export function searchDocuments(
  query: string | null,
  filters: SearchFilters = {},
  page = 0,
  size = 20,
): Promise<SearchResponse> {
  const params = new URLSearchParams()
  if (query) params.set('q', query)
  if (filters.type) params.set('type', filters.type)
  if (filters.dateFrom) params.set('dateFrom', filters.dateFrom)
  if (filters.dateTo) params.set('dateTo', filters.dateTo)
  if (filters.docDateFrom) params.set('docDateFrom', filters.docDateFrom)
  if (filters.docDateTo) params.set('docDateTo', filters.docDateTo)
  if (filters.minPages) params.set('minPages', String(filters.minPages))
  if (filters.maxPages) params.set('maxPages', String(filters.maxPages))
  params.set('page', String(page))
  params.set('size', String(size))
  return get<SearchResponse>(`/search?${params.toString()}`)
}
