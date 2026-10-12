import { Outlet, createFileRoute, redirect } from '@tanstack/react-router'
import { ErrorPanel } from '#/components/ErrorPanel'
import { ApiError, getApplication } from '#/lib/api'
import { forgetApplication, rememberApplication } from '#/lib/session'

export const Route = createFileRoute('/onboarding/$id')({
  loader: async ({ params }) => {
    try {
      const application = await getApplication(params.id)
      rememberApplication(application.id)
      return application
    } catch (error) {
      if (error instanceof ApiError && (error.status === 404 || error.status === 400)) {
        forgetApplication()
        throw redirect({ to: '/onboarding/business' })
      }
      throw error
    }
  },
  component: Outlet,
  errorComponent: ErrorPanel,
})
