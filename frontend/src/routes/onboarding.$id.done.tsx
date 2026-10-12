import { Navigate, createFileRoute, useNavigate } from '@tanstack/react-router'
import { Check } from 'lucide-react'
import { css } from 'styled-system/css'
import { Badge } from '#/components/ui/Badge'
import { Button } from '#/components/ui/Button'
import { countryName } from '#/lib/countries'
import { forgetApplication } from '#/lib/session'
import { useApplication } from '#/lib/useApplication'

export const Route = createFileRoute('/onboarding/$id/done')({ component: Done })

function Done() {
  const application = useApplication()
  const navigate = useNavigate()
  const { business } = application

  if (application.status !== 'submitted') {
    return <Navigate to="/onboarding/$id" params={{ id: application.id }} replace />
  }

  return (
    <div className={css({ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '6', py: '6', textAlign: 'center' })}>
      <span
        className={css({
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          w: '14',
          h: '14',
          borderRadius: 'full',
          bg: 'success.bg',
          border: '1px solid',
          borderColor: 'success.border',
          color: 'success.fg',
          animation: 'rise 320ms ease-out',
        })}
      >
        <Check size={26} strokeWidth={2.5} />
      </span>
      <div className={css({ display: 'flex', flexDirection: 'column', gap: '2', alignItems: 'center' })}>
        <h1 className={css({ fontSize: '2xl', fontWeight: 'semibold', letterSpacing: '-0.02em' })}>
          Application submitted
        </h1>
        {business ? (
          <p className={css({ color: 'text.secondary' })}>
            {business.legalName} · {countryName(business.country)}
          </p>
        ) : null}
      </div>
      <Badge tone="warning">In review</Badge>
      <Button
        variant="secondary"
        size="lg"
        onClick={() => {
          forgetApplication()
          void navigate({ to: '/onboarding/business' })
        }}
      >
        Start over
      </Button>
    </div>
  )
}
