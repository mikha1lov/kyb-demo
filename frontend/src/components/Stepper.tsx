import { Link, useLocation } from '@tanstack/react-router'
import { Check } from 'lucide-react'
import { css } from 'styled-system/css'

const steps = [
  { to: '/onboarding/business', label: 'Business' },
  { to: '/onboarding/documents', label: 'Documents' },
  { to: '/onboarding/review', label: 'Review' },
] as const

export function Stepper() {
  const { pathname } = useLocation()
  const current = Math.max(
    0,
    steps.findIndex((step) => pathname.startsWith(step.to)),
  )

  return (
    <ol className={css({ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '3', mb: '6', listStyle: 'none' })}>
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
            {done ? (
              <Link to={step.to} className={css({ display: 'block', _hover: { opacity: 0.85 } })}>
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
