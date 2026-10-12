import { Link, useLocation, useParams } from '@tanstack/react-router'
import { Check } from 'lucide-react'
import { css } from 'styled-system/css'

const steps = [
  { to: '/onboarding/$id/business', slug: 'business', label: 'Company' },
  { to: '/onboarding/$id/documents', slug: 'documents', label: 'Documents' },
  { to: '/onboarding/$id/details', slug: 'details', label: 'Details' },
  { to: '/onboarding/$id/review', slug: 'review', label: 'Review' },
] as const

export function Stepper() {
  const { pathname } = useLocation()
  const { id } = useParams({ strict: false })
  const current = Math.max(
    0,
    steps.findIndex((step) => pathname.endsWith(`/${step.slug}`)),
  )

  return (
    <ol className={css({ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '3', mb: '6', listStyle: 'none' })}>
      {steps.map((step, index) => {
        const done = index < current
        const active = index === current
        const content = (
          <>
            <span
              className={css({
                display: 'block',
                h: '1',
                borderRadius: 'full',
                bg: done || active ? 'text.primary' : 'border.default',
                opacity: active ? 0.55 : 1,
                transition: 'background-color 200ms, opacity 200ms',
              })}
            />
            <span
              className={css({
                display: 'flex',
                alignItems: 'center',
                gap: '1.5',
                mt: '2.5',
                fontSize: 'xs',
                fontWeight: 'medium',
                color: active ? 'text.primary' : 'text.muted',
              })}
            >
              {done ? <Check size={12} /> : null}
              {step.label}
            </span>
          </>
        )
        return (
          <li key={step.to} aria-current={active ? 'step' : undefined}>
            {done && id ? (
              <Link to={step.to} params={{ id }} className={css({ display: 'block', _hover: { opacity: 0.85 } })}>
                {content}
              </Link>
            ) : (
              content
            )}
          </li>
        )
      })}
    </ol>
  )
}
