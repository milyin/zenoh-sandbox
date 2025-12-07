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
        component: () => import('../views/RuntimeView.vue'),
        children: [
          {
            path: '',
            redirect: (to) => {
              return `/nodes/runtime/${to.params.id}/logs`
            }
          },
          {
            path: 'logs',
            name: 'nodes-runtime-logs',
            component: () => import('../views/NodesRuntimeLogs.vue'),
            props: true
          },
          {
            path: 'config',
            name: 'nodes-runtime-config',
            component: () => import('../views/NodesRuntimeConfig.vue'),
            props: true
          },
          {
            path: 'adminspace',
            name: 'nodes-runtime-adminspace',
            component: () => import('../views/NodesRuntimeAdminspace.vue'),
            props: true
          }
        ]
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
