import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
import { BusinessForm } from '#/components/BusinessForm'
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
    <BusinessForm
      initial={null}
      onSubmit={async (business) => {
        const application = await createApplication(business)
        rememberApplication(application.id)
        await navigate({ to: '/onboarding/$id/documents', params: { id: application.id } })
      }}
    />
  )
}
