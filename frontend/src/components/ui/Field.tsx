import type { InputHTMLAttributes, ReactNode } from 'react'
import { css } from 'styled-system/css'

export function Field({
  label,
  htmlFor,
  error,
  tag,
  children,
}: {
  label: string
  htmlFor?: string
  error?: string
  tag?: ReactNode
  children: ReactNode
}) {
  return (
    <div className={css({ display: 'flex', flexDirection: 'column', gap: '2' })}>
      <div className={css({ display: 'flex', alignItems: 'center', justifyContent: 'space-between', minH: '6' })}>
        <label htmlFor={htmlFor} className={css({ fontSize: 'sm', fontWeight: 'medium', color: 'text.secondary' })}>
          {label}
        </label>
        {tag}
      </div>
      {children}
      {error ? (
        <p role="alert" className={css({ fontSize: 'xs', color: 'danger.fg' })}>
          {error}
        </p>
      ) : null}
    </div>
  )
}

export const control = css({
  w: 'full',
  h: '11',
  px: '3.5',
  bg: 'bg.surface',
  border: '1px solid',
  borderColor: 'border.default',
  borderRadius: 'md',
  transition: 'border-color 120ms, box-shadow 120ms',
  _placeholder: { color: 'text.muted' },
  _hover: { borderColor: 'border.strong' },
  _focus: { outline: 'none', borderColor: 'border.strong', boxShadow: '0 0 0 3px rgba(255, 255, 255, 0.06)' },
  _invalid: { borderColor: 'danger.border' },
})

export function TextInput({ invalid, className, ...rest }: InputHTMLAttributes<HTMLInputElement> & { invalid?: boolean }) {
  return (
    <input
      data-1p-ignore
      data-lpignore="true"
      {...rest}
      aria-invalid={invalid || undefined}
      className={[control, className].filter(Boolean).join(' ')}
    />
  )
}
