import type { ReactNode } from 'react'
import { cva, type RecipeVariantProps } from 'styled-system/css'

const badge = cva({
  base: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '1.5',
    h: '6',
    px: '2.5',
    borderRadius: 'full',
    border: '1px solid',
    fontSize: 'xs',
    fontWeight: 'medium',
    whiteSpace: 'nowrap',
  },
  variants: {
    tone: {
      neutral: { bg: 'bg.raised', borderColor: 'border.default', color: 'text.secondary' },
      success: { bg: 'success.bg', borderColor: 'success.border', color: 'success.fg' },
      warning: { bg: 'warning.bg', borderColor: 'warning.border', color: 'warning.fg' },
      danger: { bg: 'danger.bg', borderColor: 'danger.border', color: 'danger.fg' },
    },
  },
  defaultVariants: { tone: 'neutral' },
})

export function Badge({ tone, children }: RecipeVariantProps<typeof badge> & { children: ReactNode }) {
  return <span className={badge({ tone })}>{children}</span>
}
