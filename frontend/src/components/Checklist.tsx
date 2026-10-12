import { Check } from 'lucide-react'
import { css } from 'styled-system/css'
import type { DocumentEntry } from '#/lib/draft'
import { satisfiedBy } from '#/lib/requirements'
import { requirementLabels, requirements } from '#/lib/schema'

export function Checklist({ documents }: { documents: DocumentEntry[] }) {
  return (
    <ul className={css({ display: 'flex', flexDirection: 'column', gap: '3', listStyle: 'none' })}>
      {requirements.map((requirement) => {
        const document = satisfiedBy(documents, requirement)
        return (
          <li key={requirement} className={css({ display: 'flex', alignItems: 'center', gap: '3', minH: '6' })}>
            <span
              className={css({
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                w: '5',
                h: '5',
                flexShrink: 0,
                borderRadius: 'full',
                border: '1px solid',
                borderColor: document ? 'success.border' : 'border.strong',
                bg: document ? 'success.bg' : 'transparent',
                color: 'success.fg',
                transition: 'background-color 200ms, border-color 200ms',
              })}
            >
              {document ? <Check size={12} strokeWidth={3} /> : null}
            </span>
            <span className={css({ fontWeight: 'medium', color: document ? 'text.primary' : 'text.secondary' })}>
              {requirementLabels[requirement]}
            </span>
            {document ? (
              <span
                className={css({
                  ml: 'auto',
                  minW: 0,
                  maxW: '45%',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                  color: 'text.muted',
                })}
              >
                {document.name}
              </span>
            ) : null}
          </li>
        )
      })}
    </ul>
  )
}
