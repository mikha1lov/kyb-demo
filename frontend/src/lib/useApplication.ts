import { getRouteApi, useRouter } from '@tanstack/react-router'

const route = getRouteApi('/onboarding/$id')

export function useApplication() {
  return route.useLoaderData()
}

export function useReload() {
  const router = useRouter()
  return () => router.invalidate()
}
