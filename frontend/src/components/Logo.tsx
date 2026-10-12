import { css } from 'styled-system/css'

export function Logo() {
  return (
    <div className={css({ display: 'flex', alignItems: 'center', gap: '3' })}>
      <svg width="28" height="28" viewBox="0 0 32 32" fill="none" aria-hidden>
        <rect width="32" height="32" rx="9" fill="#f4f4f5" />
        <circle cx="16" cy="16" r="7" stroke="#0a0a0b" strokeWidth="3" />
      </svg>
      <span className={css({ fontWeight: 'semibold', letterSpacing: '-0.01em' })}>Business verification</span>
    </div>
  )
}
