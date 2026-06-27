<script setup lang="ts">
import { ArchiveRestore, FolderOpen, Upload } from '@lucide/vue'
import ToastContainer from './components/ToastContainer.vue'
import ActivityDropdown from './components/ActivityDropdown.vue'
</script>

<template>
  <div class="layout">
    <header class="header">
      <div class="header__left">
        <RouterLink to="/" class="header__logo">
          <ArchiveRestore :size="20" class="header__logo-icon" />
          <span>Scrinium</span>
        </RouterLink>
        <nav class="header__nav">
          <RouterLink to="/" class="header__link">
            <FolderOpen :size="16" :stroke-width="1.5" />
            <span>Documents</span>
          </RouterLink>
          <RouterLink to="/upload" class="header__link">
            <Upload :size="16" :stroke-width="1.5" />
            <span>Upload</span>
          </RouterLink>
        </nav>
      </div>
      <div class="header__right">
        <ActivityDropdown />
      </div>
    </header>
    <ToastContainer />

    <main class="main">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.layout {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.header {
  height: var(--header-height);
  background-color: var(--color-bg-surface);
  border-bottom: 1px solid var(--color-border-subtle);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  flex-shrink: 0;
}

.header__right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header__left {
  display: flex;
  align-items: center;
  gap: 32px;
}

.header__logo {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.header__logo-icon {
  color: var(--color-accent);
}

.header__nav {
  display: flex;
  align-items: center;
  gap: 4px;
}

.header__link {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: var(--radius-md);
  font-size: 13px;
  color: var(--color-text-secondary);
  transition: color 0.15s, background-color 0.15s;
}

.header__link:hover {
  color: var(--color-text-primary);
  background-color: var(--color-bg-elevated);
}

.header__link.router-link-active {
  color: var(--color-accent);
  background-color: var(--color-accent-subtle);
}

.main {
  flex: 1;
  max-width: 1080px;
  width: 100%;
  margin: 0 auto;
  padding: 32px 24px;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
</style>
