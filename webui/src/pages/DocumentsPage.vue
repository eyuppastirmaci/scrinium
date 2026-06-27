<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { File as FileIcon, Upload, Search, X, SlidersHorizontal } from '@lucide/vue'
import ViewToggle from '../components/ViewToggle.vue'
import DocumentGrid from '../features/documents/DocumentGrid.vue'
import DocumentTable from '../features/documents/DocumentTable.vue'
import { fetchDocuments, getThumbnailUrl, type DocumentSummary } from '../api/documents'
import { searchDocuments, type SearchResultItem, type SearchFilters } from '../api/search'

const documents = ref<DocumentSummary[]>([])
const viewMode = ref<'grid' | 'list'>('grid')
const loading = ref(true)
const error = ref(false)
const page = ref(0)
const hasNext = ref(false)
const loadingMore = ref(false)

const searchQuery = ref('')
const searchResults = ref<SearchResultItem[]>([])
const searchTotalCount = ref(0)
const searching = ref(false)
const searchError = ref(false)
const isSearchActive = ref(false)

const showFilters = ref(false)
const filterType = ref('')
const filterDateFrom = ref('')
const filterDateTo = ref('')
const filterMinPages = ref('')
const filterMaxPages = ref('')

const activeFilters = computed<SearchFilters>(() => {
  const f: SearchFilters = {}
  if (filterType.value) f.type = filterType.value
  if (filterDateFrom.value) f.dateFrom = filterDateFrom.value
  if (filterDateTo.value) f.dateTo = filterDateTo.value
  if (filterMinPages.value) f.minPages = parseInt(filterMinPages.value)
  if (filterMaxPages.value) f.maxPages = parseInt(filterMaxPages.value)
  return f
})

const hasActiveFilters = computed(() => Object.keys(activeFilters.value).length > 0)

const activeFilterCount = computed(() => Object.keys(activeFilters.value).length)

let debounceTimer: ReturnType<typeof setTimeout> | null = null

watch(searchQuery, () => scheduleSearch())
watch(activeFilters, () => scheduleSearch(), { deep: true })

function scheduleSearch() {
  if (debounceTimer) clearTimeout(debounceTimer)
  const q = searchQuery.value.trim()
  if (!q && !hasActiveFilters.value) {
    isSearchActive.value = false
    searchResults.value = []
    searchError.value = false
    return
  }
  debounceTimer = setTimeout(() => performSearch(), 300)
}

async function performSearch() {
  searching.value = true
  searchError.value = false
  isSearchActive.value = true
  try {
    const q = searchQuery.value.trim() || null
    const data = await searchDocuments(q, activeFilters.value)
    searchResults.value = data.items
    searchTotalCount.value = data.totalCount
  } catch {
    searchError.value = true
  } finally {
    searching.value = false
  }
}

function clearSearch() {
  searchQuery.value = ''
  clearFilters()
}

function clearFilters() {
  filterType.value = ''
  filterDateFrom.value = ''
  filterDateTo.value = ''
  filterMinPages.value = ''
  filterMaxPages.value = ''
}

function removeFilter(key: string) {
  switch (key) {
    case 'type': filterType.value = ''; break
    case 'dateFrom': filterDateFrom.value = ''; break
    case 'dateTo': filterDateTo.value = ''; break
    case 'minPages': filterMinPages.value = ''; break
    case 'maxPages': filterMaxPages.value = ''; break
  }
}

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

    <template v-else-if="documents.length || isSearchActive">
      <div class="docs-page__toolbar">
        <h2 class="docs-page__title">Documents</h2>
        <div class="docs-page__actions">
          <div class="search-bar">
            <Search :size="15" :stroke-width="2" class="search-bar__icon" />
            <input
              v-model="searchQuery"
              type="text"
              class="search-bar__input"
              placeholder="Search documents..."
            />
            <button
              v-if="searchQuery || hasActiveFilters"
              class="search-bar__clear"
              @click="clearSearch"
            >
              <X :size="14" :stroke-width="2" />
            </button>
          </div>
          <button
            class="filter-toggle"
            :class="{ 'filter-toggle--active': showFilters || hasActiveFilters }"
            @click="showFilters = !showFilters"
          >
            <SlidersHorizontal :size="15" :stroke-width="2" />
            <span v-if="activeFilterCount" class="filter-toggle__badge">{{ activeFilterCount }}</span>
          </button>
          <ViewToggle v-model="viewMode" />
        </div>
      </div>

      <!-- Filter bar -->
      <div v-if="showFilters" class="filter-bar">
        <div class="filter-bar__group">
          <label class="filter-bar__label">Type</label>
          <select v-model="filterType" class="filter-bar__select">
            <option value="">All</option>
            <option value="application/pdf">PDF</option>
            <option value="image">Image</option>
          </select>
        </div>
        <div class="filter-bar__group">
          <label class="filter-bar__label">Uploaded after</label>
          <input v-model="filterDateFrom" type="date" class="filter-bar__date" />
        </div>
        <div class="filter-bar__group">
          <label class="filter-bar__label">Uploaded before</label>
          <input v-model="filterDateTo" type="date" class="filter-bar__date" />
        </div>
        <div class="filter-bar__group">
          <label class="filter-bar__label">Min pages</label>
          <input v-model="filterMinPages" type="number" min="1" class="filter-bar__number" placeholder="—" />
        </div>
        <div class="filter-bar__group">
          <label class="filter-bar__label">Max pages</label>
          <input v-model="filterMaxPages" type="number" min="1" class="filter-bar__number" placeholder="—" />
        </div>
        <button v-if="hasActiveFilters" class="filter-bar__clear" @click="clearFilters">
          Clear filters
        </button>
      </div>

      <!-- Active filter chips -->
      <div v-if="hasActiveFilters && !showFilters" class="filter-chips">
        <span v-if="activeFilters.type" class="filter-chip">
          {{ activeFilters.type === 'application/pdf' ? 'PDF' : 'Image' }}
          <button class="filter-chip__remove" @click="removeFilter('type')"><X :size="12" /></button>
        </span>
        <span v-if="activeFilters.dateFrom" class="filter-chip">
          From {{ activeFilters.dateFrom }}
          <button class="filter-chip__remove" @click="removeFilter('dateFrom')"><X :size="12" /></button>
        </span>
        <span v-if="activeFilters.dateTo" class="filter-chip">
          To {{ activeFilters.dateTo }}
          <button class="filter-chip__remove" @click="removeFilter('dateTo')"><X :size="12" /></button>
        </span>
        <span v-if="activeFilters.minPages" class="filter-chip">
          ≥{{ activeFilters.minPages }} pages
          <button class="filter-chip__remove" @click="removeFilter('minPages')"><X :size="12" /></button>
        </span>
        <span v-if="activeFilters.maxPages" class="filter-chip">
          ≤{{ activeFilters.maxPages }} pages
          <button class="filter-chip__remove" @click="removeFilter('maxPages')"><X :size="12" /></button>
        </span>
      </div>

      <!-- Search results -->
      <template v-if="isSearchActive">
        <div v-if="searching" class="docs-page__status">Searching...</div>
        <div v-else-if="searchError" class="docs-page__error">Search failed.</div>
        <div v-else-if="!searchResults.length" class="docs-page__empty-search">
          No results for "{{ searchQuery }}"
        </div>
        <template v-else>
          <p class="search-results__count">
            {{ searchTotalCount }} result{{ searchTotalCount === 1 ? '' : 's' }} for "{{ searchQuery }}"
          </p>
          <div class="search-results">
            <RouterLink
              v-for="result in searchResults"
              :key="result.documentId"
              :to="{ name: 'document-detail', params: { id: result.documentId } }"
              class="search-results__item"
            >
              <img
                :src="getThumbnailUrl(result.documentId, 'small')"
                alt=""
                class="search-results__thumb"
                @error="($event.target as HTMLImageElement).style.display = 'none'"
              />
              <div class="search-results__body">
                <div class="search-results__header">
                  <span class="search-results__name">{{ result.fileName }}</span>
                  <span class="search-results__score">{{ Math.round(result.score * 100) }}%</span>
                </div>
                <p class="search-results__snippet" v-html="result.snippet" />
              </div>
            </RouterLink>
          </div>
        </template>
      </template>

      <!-- Normal document list -->
      <template v-else>
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

.docs-page__actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-md);
  background-color: var(--color-bg-surface);
  transition: border-color 0.15s;
}

.search-bar:focus-within {
  border-color: var(--color-accent);
}

.search-bar__icon {
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

.search-bar__input {
  border: none;
  outline: none;
  background: transparent;
  font-size: 13px;
  color: var(--color-text-primary);
  width: 200px;
}

.search-bar__input::placeholder {
  color: var(--color-text-tertiary);
}

.search-bar__clear {
  color: var(--color-text-tertiary);
  flex-shrink: 0;
  transition: color 0.15s;
}

.search-bar__clear:hover {
  color: var(--color-text-primary);
}

.filter-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
  width: 34px;
  height: 34px;
  border-radius: var(--radius-md);
  color: var(--color-text-tertiary);
  border: 1px solid var(--color-border-default);
  transition: color 0.15s, border-color 0.15s, background-color 0.15s;
}

.filter-toggle:hover {
  color: var(--color-text-primary);
  background-color: var(--color-bg-elevated);
}

.filter-toggle--active {
  color: var(--color-accent);
  border-color: var(--color-accent);
}

.filter-toggle__badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 16px;
  height: 16px;
  font-size: 10px;
  font-weight: 600;
  line-height: 16px;
  text-align: center;
  color: #fff;
  background-color: var(--color-accent);
  border-radius: var(--radius-pill);
}

.filter-bar {
  display: flex;
  align-items: flex-end;
  gap: 16px;
  padding: 14px 16px;
  margin-bottom: 16px;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-surface);
  flex-wrap: wrap;
}

.filter-bar__group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.filter-bar__label {
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.filter-bar__select,
.filter-bar__date,
.filter-bar__number {
  padding: 6px 10px;
  font-size: 12px;
  color: var(--color-text-primary);
  background-color: var(--color-bg-base);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color 0.15s;
}

.filter-bar__select:focus,
.filter-bar__date:focus,
.filter-bar__number:focus {
  border-color: var(--color-accent);
}

.filter-bar__number {
  width: 70px;
}

.filter-bar__clear {
  font-size: 12px;
  color: var(--color-text-tertiary);
  padding: 6px 0;
  transition: color 0.15s;
}

.filter-bar__clear:hover {
  color: var(--color-accent);
}

.filter-chips {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 14px;
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  font-size: 11px;
  font-weight: 500;
  color: var(--color-accent);
  background-color: var(--color-accent-subtle);
  border-radius: var(--radius-pill);
}

.filter-chip__remove {
  color: var(--color-accent);
  opacity: 0.6;
  transition: opacity 0.15s;
}

.filter-chip__remove:hover {
  opacity: 1;
}

.docs-page__empty-search {
  text-align: center;
  padding: 48px 0;
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.search-results__count {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-bottom: 12px;
}

.search-results {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-surface);
  overflow: hidden;
}

.search-results__item {
  display: flex;
  gap: 14px;
  padding: 16px;
  border-bottom: 1px solid var(--color-border-subtle);
  transition: background-color 0.15s;
}

.search-results__item:last-child {
  border-bottom: none;
}

.search-results__item:hover {
  background-color: var(--color-bg-elevated);
}

.search-results__thumb {
  width: 48px;
  height: 48px;
  object-fit: cover;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

.search-results__body {
  flex: 1;
  min-width: 0;
}

.search-results__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 6px;
}

.search-results__name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.search-results__score {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-accent);
  background-color: var(--color-accent-subtle);
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  flex-shrink: 0;
}

.search-results__snippet {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.6;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.search-results__snippet :deep(mark) {
  background-color: var(--color-accent-subtle);
  color: var(--color-accent);
  border-radius: 2px;
  padding: 0 2px;
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
