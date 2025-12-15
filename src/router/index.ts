import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('../views/MainView.vue'),
    children: [
      {
        path: '',
        redirect: '/nodes'
      },
      {
        path: 'nodes',
        name: 'nodes',
        redirect: '/nodes/config/0/edit'
      },
      {
        path: 'nodes/config',
        name: 'nodes-config-redirect',
        redirect: () => {
          // TODO: Get first config ID and redirect to /nodes/config/0/edit
          return '/nodes/config/0/edit'
        }
      },
      {
        path: 'nodes/config/:id',
        redirect: (to) => {
          return `/nodes/config/${to.params.id}/edit`
        }
      },
      {
        path: 'nodes/config/:id/edit',
        name: 'nodes-config-edit',
        component: () => import('../views/ConfigView.vue'),
        props: true
      },
      {
        path: 'nodes/config/:id/activity',
        name: 'nodes-config-activity',
        component: () => import('../views/ConfigView.vue'),
        props: true
      },
      {
        path: 'nodes/runtime',
        name: 'nodes-runtime-redirect',
        redirect: () => {
          // TODO: Get first runtime ID from store/state
          // For now, redirect to config
          return '/nodes/config/0/edit'
        }
      },
      {
        path: 'nodes/runtime/:id',
        name: 'nodes-runtime',
        component: () => import('../views/RuntimeView.vue'),
        props: true
      },
      {
        // Support direct tab URL navigation
        path: 'nodes/runtime/:id/:tab',
        name: 'nodes-runtime-tab',
        component: () => import('../views/RuntimeView.vue'),
        props: true
      },
      {
        path: 'sessions',
        name: 'sessions',
        // Sessions content is rendered by SessionsSection, this is just a placeholder
        component: { template: '<div></div>' }
      }
    ]
  }
]

const router = createRouter({
  // Using hash mode for better compatibility with Tauri
  history: createWebHashHistory(),
  routes
})

export default router
