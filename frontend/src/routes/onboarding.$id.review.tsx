import { Navigate, createFileRoute, useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { css } from 'styled-system/css'
import { Row, Section } from '#/components/Summary'
import { StepLayout } from '#/components/StepLayout'
import { Badge } from '#/components/ui/Badge'
import { Button } from '#/components/ui/Button'
import { submitApplication } from '#/lib/api'
import { countries, flag } from '#/lib/countries'
import { messageOf } from '#/lib/errors'
import { entityTypeLabels, requirementLabels } from '#/lib/schema'
import { useApplication, useReload } from '#/lib/useApplication'

export const Route = createFileRoute('/onboarding/$id/review')({ component: ReviewStep })

function FromDocument() {
  return <Badge>From document</Badge>
}

function ReviewStep() {
  const application = useApplication()
  const reload = useReload()
  const navigate = useNavigate()
  const [submitting, setSubmitting] = useState(false)
  const [failure, setFailure] = useState<string | null>(null)
  const { business, sources } = application
  const params = { id: application.id }

  if (application.status === 'submitted') return <Navigate to="/onboarding/$id/done" params={params} replace />
  if (!business) return <Navigate to="/onboarding/$id/business" params={params} replace />
  if (!application.requirementsMet) return <Navigate to="/onboarding/$id/documents" params={params} replace />
  if (!application.businessComplete) return <Navigate to="/onboarding/$id/details" params={params} replace />

  const country = countries.find((c) => c.code === business.country)

  const submit = async () => {
    setSubmitting(true)
    setFailure(null)
    try {
      await submitApplication(application.id)
      await reload()
      await navigate({ to: '/onboarding/$id/done', params })
    } catch (error) {
      setFailure(messageOf(error))
      setSubmitting(false)
    }
  }

  return (
    <StepLayout
      title="Review"
      footer={
        <>
          <Button variant="secondary" size="lg" onClick={() => navigate({ to: '/onboarding/$id/details', params })}>
            Back
          </Button>
          <Button size="lg" className={css({ flex: 1 })} loading={submitting} onClick={submit}>
            Submit
          </Button>
        </>
      }
    >
      <Section title="Business" to="/onboarding/$id/details" id={application.id}>
        <Row label="Country">
          <span aria-hidden>{flag(business.country)}</span>
          {country?.name ?? business.country}
        </Row>
        <Row label="Entity type">{entityTypeLabels[business.entityType]}</Row>
        <Row label="Legal name" tag={sources.legalName ? <FromDocument /> : null}>
          {business.legalName}
        </Row>
        <Row label="Registration number" tag={sources.registrationNumber ? <FromDocument /> : null}>
          {business.registrationNumber}
        </Row>
        <Row label="Registered address" tag={sources.address ? <FromDocument /> : null}>
          {business.address}
        </Row>
      </Section>
      <Section title="Documents" to="/onboarding/$id/documents" id={application.id}>
        {application.checklist.map((item) => (
          <Row key={item.requirement} label={requirementLabels[item.requirement]}>
            {application.documents.find((d) => d.id === item.documentId)?.name}
          </Row>
        ))}
      </Section>
      {failure ? (
        <p role="alert" className={css({ color: 'danger.fg', fontSize: 'xs' })}>
          {failure}
        </p>
      ) : null}
    </StepLayout>
  )
}
