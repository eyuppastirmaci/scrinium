import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'documents',
      component: () => import('./pages/DocumentsPage.vue'),
    },
    {
      path: '/documents/:id',
      name: 'document-detail',
      component: () => import('./pages/DocumentDetailPage.vue'),
      props: true,
    },
    {
      path: '/upload',
      name: 'upload',
      component: () => import('./pages/UploadPage.vue'),
    },
  ],
})

export default router
