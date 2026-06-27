import { get } from './client'

export interface SearchResultItem {
  documentId: string
  fileName: string
  snippet: string
  score: number
}

export interface SearchResponse {
  query: string
  items: SearchResultItem[]
  totalCount: number
  page: number
  size: number
}

export function searchDocuments(query: string, page = 0, size = 20): Promise<SearchResponse> {
  return get<SearchResponse>(`/search?q=${encodeURIComponent(query)}&page=${page}&size=${size}`)
}
