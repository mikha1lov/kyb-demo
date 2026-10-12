import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
import { useSelector } from '@tanstack/react-store'
import { css } from 'styled-system/css'
import { Row, Section } from '#/components/Summary'
import { StepLayout } from '#/components/StepLayout'
import { Badge } from '#/components/ui/Badge'
import { Button } from '#/components/ui/Button'
import { countries, flag } from '#/lib/countries'
import { draft, submit } from '#/lib/draft'
import { missing, satisfiedBy } from '#/lib/requirements'
import { entityTypeLabels, requirementLabels, requirements } from '#/lib/schema'

export const Route = createFileRoute('/onboarding/review')({
  beforeLoad: () => {
    const { business, documents } = draft.get()
    if (!business) throw redirect({ to: '/onboarding/business' })
    if (missing(documents).length > 0) throw redirect({ to: '/onboarding/documents' })
  },
  component: ReviewStep,
})

function FromDocument() {
  return <Badge>From document</Badge>
}

function ReviewStep() {
  const navigate = useNavigate()
  const business = useSelector(draft, (s) => s.business)
  const sources = useSelector(draft, (s) => s.sources)
  const documents = useSelector(draft, (s) => s.documents)

  if (!business) return null
  const country = countries.find((c) => c.code === business.country)

  return (
    <StepLayout
      title="Review"
      footer={
        <>
          <Button variant="secondary" size="lg" onClick={() => navigate({ to: '/onboarding/documents' })}>
            Back
          </Button>
          <Button
            size="lg"
            className={css({ flex: 1 })}
            onClick={() => {
              submit()
              void navigate({ to: '/onboarding/done' })
            }}
          >
            Submit
          </Button>
        </>
      }
    >
      <Section title="Business" to="/onboarding/business">
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
      <Section title="Documents" to="/onboarding/documents">
        {requirements.map((requirement) => (
          <Row key={requirement} label={requirementLabels[requirement]}>
            {satisfiedBy(documents, requirement)?.name}
          </Row>
        ))}
      </Section>
    </StepLayout>
  )
}
