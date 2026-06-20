<script setup lang="ts">
import { ArchiveRestore } from '@lucide/vue'
import UploadPage from './pages/UploadPage.vue'
</script>

<template>
  <div class="layout">
    <header class="header">
      <div class="header__logo">
        <ArchiveRestore :size="20" class="header__icon" />
        <span>Scrinium</span>
      </div>
    </header>

    <main class="main">
      <UploadPage />
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
  padding: 0 24px;
  flex-shrink: 0;
}

.header__logo {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.header__icon {
  color: var(--color-accent);
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
