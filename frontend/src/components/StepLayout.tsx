import type { ReactNode } from 'react'
import { css } from 'styled-system/css'

export function StepLayout({ title, children, footer }: { title: string; children: ReactNode; footer: ReactNode }) {
  return (
    <div className={css({ display: 'flex', flexDirection: 'column', gap: '6' })}>
      <h1 className={css({ fontSize: '2xl', fontWeight: 'semibold', letterSpacing: '-0.02em', lineHeight: '1.2' })}>
        {title}
      </h1>
      <div className={css({ display: 'flex', flexDirection: 'column', gap: '5' })}>{children}</div>
      <div className={css({ display: 'flex', gap: '3', pt: '2' })}>{footer}</div>
    </div>
  )
}
