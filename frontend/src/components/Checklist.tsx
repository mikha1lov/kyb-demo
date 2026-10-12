import { Check } from 'lucide-react'
import { css } from 'styled-system/css'
import { requirementLabels } from '#/lib/schema'
import type { Application } from '#/lib/types'

export function Checklist({ application }: { application: Application }) {
  return (
    <ul className={css({ display: 'flex', flexDirection: 'column', gap: '3', listStyle: 'none' })}>
      {application.checklist.map((item) => {
        const document = application.documents.find((d) => d.id === item.documentId)
        return (
          <li key={item.requirement} className={css({ display: 'flex', alignItems: 'center', gap: '3', minH: '6' })}>
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
              {requirementLabels[item.requirement]}
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
