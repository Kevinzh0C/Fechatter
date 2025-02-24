export const debugRoutes = [
  {
    path: '/debug',
    name: 'Debug',
    component: () => import('../../views/Debug.vue'),
    meta: { requiresAuth: false }
  },
  {
    path: '/debug/protobuf-analytics',
    name: 'ProtobufAnalyticsTest',
    component: () => import('../../components/debug/ProtobufAnalyticsTest.vue'),
    meta: { requiresAuth: false }
  },
  {
    path: '/debug/analytics',
    name: 'AnalyticsTest',
    component: () => import('@/views/AnalyticsTest.vue'),
    meta: {
      title: 'Analytics Testing',
      requiresAuth: true,
      debugOnly: true
    }
  }
] 