import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

export const ROUTE_NAMES = {
  HOME: 'home',
  ABOUT: 'about',
  ANALYSIS: 'analysis',
  ANALYZE_AGAIN: 're-analyze',
  ANALYSIS_LIST: 'analysis-last',
  ANALYSIS_NOT_FOUND: 'analysis-not-found',
  SWAGGER_UI: 'swagger-ui',
}

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: ROUTE_NAMES.HOME,
      component: HomeView,
    },
    {
      path: '/about',
      name: ROUTE_NAMES.ABOUT,
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/AboutView.vue'),
    },
    {
      path: '/analysis-not-found',
      name: ROUTE_NAMES.ANALYSIS_NOT_FOUND,
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/AnalysisNotFoundView.vue'),
    },
    {
      path: '/analyze/again/:uuid(\\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\\b)',
      name: ROUTE_NAMES.ANALYZE_AGAIN,
      component: () => import('../views/AnalyzeAgainView.vue'),
      props: (route) => ({
        uuid: route.params.uuid,
      }),
    },
    {
      path: '/analysis/:uuid(\\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\\b)',
      name: ROUTE_NAMES.ANALYSIS,
      component: () => import('../views/AnalysisView.vue'),
      props: (route) => ({
        uuid: route.params.uuid,
      }),
    },
    {
      path: '/analysis/list',
      name: ROUTE_NAMES.ANALYSIS_LIST,
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/AnalysisListView.vue'),
    },
    {
      path: '/openapi',
      name: ROUTE_NAMES.SWAGGER_UI,
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/SwaggerUIView.vue'),
    },
    // Catch-all route (404)
    {
      path: '/:pathMatch(.*)*',
      name: 'not-found',
      component: () => import('../views/NotFoundView.vue'),
    },
  ],
})

export default router
