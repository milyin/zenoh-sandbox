import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('../views/HomeView.vue')
  },
  {
    path: '/nodes',
    component: () => import('../views/NodesView.vue'),
    children: [
      {
        path: '',
        name: 'nodes',
        component: () => import('../views/NodesActivityLog.vue')
      },
      {
        path: 'config',
        name: 'nodes-config-redirect',
        redirect: () => {
          // TODO: Get first config ID and redirect to /nodes/config/0/edit
          return '/nodes/config/0/edit'
        }
      },
      {
        path: 'config/:id',
        redirect: (to) => {
          return `/nodes/config/${to.params.id}/edit`
        }
      },
      {
        path: 'config/:id/edit',
        name: 'nodes-config-edit',
        component: () => import('../views/NodesConfigEdit.vue'),
        props: true
      },
      {
        path: 'runtime',
        name: 'nodes-runtime-redirect',
        redirect: () => {
          // TODO: Get first runtime ID from store/state
          // For now, redirect to activity log
          return '/nodes'
        }
      },
      {
        path: 'runtime/:id',
        name: 'nodes-runtime',
        component: () => import('../views/RuntimeView.vue'),
        props: true
      },
      {
        // Support direct tab URL navigation
        path: 'runtime/:id/:tab',
        name: 'nodes-runtime-tab',
        component: () => import('../views/RuntimeView.vue'),
        props: true
      }
    ]
  },
  {
    path: '/sessions',
    name: 'sessions',
    component: () => import('../views/SessionsView.vue')
  }
]

const router = createRouter({
  // Using hash mode for better compatibility with Tauri
  history: createWebHashHistory(),
  routes
})

export default router
