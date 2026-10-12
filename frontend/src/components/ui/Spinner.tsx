import { css } from 'styled-system/css'

export function Spinner({ size = 16 }: { size?: number }) {
  return (
    <svg
      className={css({ animation: 'spin 0.8s linear infinite', flexShrink: 0 })}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden
    >
      <circle cx="12" cy="12" r="9" stroke="currentColor" strokeOpacity="0.25" strokeWidth="3" />
      <path d="M21 12a9 9 0 0 0-9-9" stroke="currentColor" strokeWidth="3" strokeLinecap="round" />
    </svg>
  )
}
