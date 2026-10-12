import { Navigate, createFileRoute, useNavigate } from '@tanstack/react-router'
import { CompanyForm } from '#/components/CompanyForm'
import { saveBusiness } from '#/lib/api'
import { useApplication, useReload } from '#/lib/useApplication'

export const Route = createFileRoute('/onboarding/$id/business')({ component: EditCompany })

function EditCompany() {
  const application = useApplication()
  const reload = useReload()
  const navigate = useNavigate()
  const { business } = application

  if (application.status === 'submitted') {
    return <Navigate to="/onboarding/$id/done" params={{ id: application.id }} replace />
  }
  if (!business) return null

  return (
    <CompanyForm
      initial={{ country: business.country, entityType: business.entityType }}
      onSubmit={async (company) => {
        await saveBusiness(application.id, { ...business, ...company }, application.sources)
        await reload()
        await navigate({ to: '/onboarding/$id/documents', params: { id: application.id } })
      }}
    />
  )
}
