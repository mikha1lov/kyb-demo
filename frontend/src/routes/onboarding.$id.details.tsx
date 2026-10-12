import { Navigate, createFileRoute, useNavigate } from '@tanstack/react-router'
import { DetailsForm } from '#/components/DetailsForm'
import { saveBusiness } from '#/lib/api'
import { sourcesFor } from '#/lib/sources'
import { useApplication, useReload } from '#/lib/useApplication'

export const Route = createFileRoute('/onboarding/$id/details')({ component: DetailsStep })

function DetailsStep() {
  const application = useApplication()
  const reload = useReload()
  const navigate = useNavigate()
  const { business, suggestions } = application
  const params = { id: application.id }

  if (application.status === 'submitted') return <Navigate to="/onboarding/$id/done" params={params} replace />
  if (!business) return <Navigate to="/onboarding/$id/business" params={params} replace />
  if (!application.requirementsMet) return <Navigate to="/onboarding/$id/documents" params={params} replace />

  const initial = {
    legalName: business.legalName || suggestions.legalName?.value || '',
    registrationNumber: business.registrationNumber || suggestions.registrationNumber?.value || '',
    address: business.address || suggestions.address?.value || '',
  }

  return (
    <DetailsForm
      initial={initial}
      suggestions={suggestions}
      onBack={() => void navigate({ to: '/onboarding/$id/documents', params })}
      onSubmit={async (details) => {
        await saveBusiness(application.id, { ...business, ...details }, sourcesFor(details, application))
        await reload()
        await navigate({ to: '/onboarding/$id/review', params })
      }}
    />
  )
}
