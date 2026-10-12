import { css } from 'styled-system/css'

type Option<T extends string> = { value: T; label: string }

export function Segmented<T extends string>({
  name,
  value,
  options,
  onChange,
}: {
  name: string
  value: T
  options: readonly Option<T>[]
  onChange: (value: T) => void
}) {
  return (
    <div role="radiogroup" className={css({ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '2' })}>
      {options.map((option) => (
        <label key={option.value} className={css({ position: 'relative', display: 'block' })}>
          <input
            type="radio"
            name={name}
            value={option.value}
            checked={value === option.value}
            onChange={() => onChange(option.value)}
            className={`peer ${css({ position: 'absolute', opacity: 0, inset: 0, cursor: 'pointer' })}`}
          />
          <span
            className={css({
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              h: '11',
              px: '3',
              bg: 'bg.surface',
              border: '1px solid',
              borderColor: 'border.default',
              borderRadius: 'md',
              color: 'text.secondary',
              fontWeight: 'medium',
              transition: 'background-color 120ms, border-color 120ms, color 120ms',
              _peerHover: { borderColor: 'border.strong' },
              _peerChecked: { bg: 'bg.raised', borderColor: 'text.secondary', color: 'text.primary' },
              _peerFocusVisible: { outline: '2px solid', outlineColor: 'focus', outlineOffset: '2px' },
            })}
          >
            {option.label}
          </span>
        </label>
      ))}
    </div>
  )
}
