import type { ButtonHTMLAttributes } from 'react'
import { cva, type RecipeVariantProps } from 'styled-system/css'
import { Spinner } from './Spinner'

const button = cva({
  base: {
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '2',
    flexShrink: 0,
    fontWeight: 'medium',
    borderRadius: 'md',
    border: '1px solid transparent',
    whiteSpace: 'nowrap',
    transition: 'background-color 120ms, border-color 120ms, color 120ms, opacity 120ms',
    _disabled: { opacity: 0.4, cursor: 'not-allowed' },
  },
  variants: {
    variant: {
      primary: { bg: 'action.primary', color: 'text.inverse', _hover: { _enabled: { bg: 'action.primaryHover' } } },
      secondary: {
        bg: 'bg.raised',
        borderColor: 'border.default',
        _hover: { _enabled: { bg: 'bg.hover', borderColor: 'border.strong' } },
      },
      ghost: { color: 'text.secondary', _hover: { _enabled: { color: 'text.primary', bg: 'bg.raised' } } },
    },
    size: {
      sm: { h: '8', px: '3', fontSize: 'sm' },
      md: { h: '10', px: '4', fontSize: 'sm' },
      lg: { h: '12', px: '6', fontSize: 'md' },
    },
  },
  defaultVariants: { variant: 'primary', size: 'md' },
})

type Props = ButtonHTMLAttributes<HTMLButtonElement> & RecipeVariantProps<typeof button> & { loading?: boolean }

export function Button({ variant, size, loading, disabled, children, className, ...rest }: Props) {
  return (
    <button
      type="button"
      {...rest}
      disabled={disabled || loading}
      className={[button({ variant, size }), className].filter(Boolean).join(' ')}
    >
      {loading ? <Spinner /> : null}
      {children}
    </button>
  )
}
