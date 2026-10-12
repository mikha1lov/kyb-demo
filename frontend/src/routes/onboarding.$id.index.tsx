import { Navigate, createFileRoute } from '@tanstack/react-router'
import { useApplication } from '#/lib/useApplication'

export const Route = createFileRoute('/onboarding/$id/')({ component: Resume })

function Resume() {
  const application = useApplication()
  const params = { id: application.id }

  if (application.status === 'submitted') return <Navigate to="/onboarding/$id/done" params={params} replace />
  if (!application.business) return <Navigate to="/onboarding/$id/business" params={params} replace />
  if (application.complete) return <Navigate to="/onboarding/$id/review" params={params} replace />
  return <Navigate to="/onboarding/$id/documents" params={params} replace />
}
