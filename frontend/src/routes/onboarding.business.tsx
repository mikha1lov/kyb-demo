import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
import { CompanyForm } from '#/components/CompanyForm'
import { createApplication } from '#/lib/api'
import { currentApplication, rememberApplication } from '#/lib/session'

export const Route = createFileRoute('/onboarding/business')({
  beforeLoad: () => {
    const id = currentApplication()
    if (id) throw redirect({ to: '/onboarding/$id', params: { id } })
  },
  component: NewApplication,
})

function NewApplication() {
  const navigate = useNavigate()

  return (
    <CompanyForm
      initial={null}
      onSubmit={async (company) => {
        const application = await createApplication({ ...company, legalName: '', registrationNumber: '', address: '' })
        rememberApplication(application.id)
        await navigate({ to: '/onboarding/$id/documents', params: { id: application.id } })
      }}
    />
  )
}
