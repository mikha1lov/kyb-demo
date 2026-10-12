import { Navigate, createFileRoute, useNavigate } from '@tanstack/react-router'
import { BusinessForm } from '#/components/BusinessForm'
import { saveBusiness } from '#/lib/api'
import { keepSources } from '#/lib/sources'
import { useApplication, useReload } from '#/lib/useApplication'

export const Route = createFileRoute('/onboarding/$id/business')({ component: EditBusiness })

function EditBusiness() {
  const application = useApplication()
  const reload = useReload()
  const navigate = useNavigate()

  if (application.status === 'submitted') {
    return <Navigate to="/onboarding/$id/done" params={{ id: application.id }} replace />
  }

  return (
    <BusinessForm
      initial={application.business}
      onSubmit={async (business) => {
        const sources = keepSources(application.business, business, application.sources)
        await saveBusiness(application.id, business, sources)
        await reload()
        await navigate({ to: '/onboarding/$id/documents', params: { id: application.id } })
      }}
    />
  )
}
