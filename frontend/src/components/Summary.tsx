import { Link } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { css } from 'styled-system/css'

export function Section({
  title,
  to,
  id,
  children,
}: {
  title: string
  to: '/onboarding/$id/details' | '/onboarding/$id/documents'
  id: string
  children: ReactNode
}) {
  return (
    <section className={css({ display: 'flex', flexDirection: 'column' })}>
      <div className={css({ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: '2' })}>
        <h2 className={css({ fontSize: 'md', fontWeight: 'semibold' })}>{title}</h2>
        <Link
          to={to}
          params={{ id }}
          className={css({ color: 'text.secondary', fontWeight: 'medium', _hover: { color: 'text.primary' } })}
        >
          Edit
        </Link>
      </div>
      <dl
        className={css({
          display: 'flex',
          flexDirection: 'column',
          borderRadius: 'lg',
          bg: 'bg.raised',
          border: '1px solid',
          borderColor: 'border.subtle',
        })}
      >
        {children}
      </dl>
    </section>
  )
}

export function Row({ label, children, tag }: { label: string; children: ReactNode; tag?: ReactNode }) {
  return (
    <div
      className={css({
        display: 'grid',
        gridTemplateColumns: { base: '1fr', sm: '172px 1fr' },
        gap: { base: '1', sm: '4' },
        px: '4',
        py: '3.5',
        '&:not(:first-child)': { borderTop: '1px solid {colors.border.subtle}' },
      })}
    >
      <dt className={css({ color: 'text.muted' })}>{label}</dt>
      <dd className={css({ display: 'flex', alignItems: 'center', gap: '2', flexWrap: 'wrap', wordBreak: 'break-word' })}>
        {children}
        {tag}
      </dd>
    </div>
  )
}
