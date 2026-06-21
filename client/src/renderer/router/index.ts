import { createRouter, createWebHashHistory } from 'vue-router'
import { routes } from './routes'
import { resolveRouteAccess } from './guard'

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

router.beforeEach((to) => resolveRouteAccess(to.meta))

export default router
