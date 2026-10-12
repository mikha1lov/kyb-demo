import { CircleCheck, FileText, Trash2, TriangleAlert } from 'lucide-react'
import type { ReactNode } from 'react'
import { css } from 'styled-system/css'
import { Badge } from '#/components/ui/Badge'
import { Button } from '#/components/ui/Button'
import { Spinner } from '#/components/ui/Spinner'
import { formatBytes, formatDate } from '#/lib/format'
import { counted, isBlocking } from '#/lib/requirements'
import { requirementLabels, type Business } from '#/lib/schema'
import {
  docKindLabels,
  type DocumentView,
  type ExtractedField,
  type FieldCheck,
  type SourceField,
} from '#/lib/types'

const fieldLabels: Record<SourceField, string> = {
  legalName: 'Name',
  registrationNumber: 'Number',
  address: 'Address',
}

const useLabels: Record<SourceField, string> = {
  legalName: 'Use document name',
  registrationNumber: 'Use document number',
  address: 'Use document address',
}

const frame = css({
  display: 'flex',
  flexDirection: 'column',
  gap: '4',
  p: '4',
  borderRadius: 'lg',
  bg: 'bg.raised',
  border: '1px solid',
  borderColor: 'border.subtle',
  animation: 'rise 200ms ease-out',
})

function Highlight({ text, tokens }: { text: string; tokens: string[] }) {
  if (tokens.length === 0) return <>{text}</>
  return (
    <>
      {text.split(/(\s+)/).map((part, index) =>
        tokens.includes(part.toLowerCase().replace(/[^a-z0-9]/g, '')) ? (
          <mark
            key={index}
            className={css({ bg: 'warning.bg', color: 'warning.fg', borderRadius: 'xs', px: '1', mx: '-1' })}
          >
            {part}
          </mark>
        ) : (
          part
        ),
      )}
    </>
  )
}

function Header({
  name,
  caption,
  status,
  onRemove,
}: {
  name: string
  caption: string
  status: ReactNode
  onRemove?: () => void
}) {
  return (
    <header className={css({ display: 'flex', alignItems: 'center', gap: '3' })}>
      <span
        className={css({
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          w: '9',
          h: '9',
          flexShrink: 0,
          borderRadius: 'sm',
          bg: 'bg.surface',
          border: '1px solid',
          borderColor: 'border.subtle',
          color: 'text.secondary',
        })}
      >
        <FileText size={18} />
      </span>
      <div className={css({ minW: 0, flex: 1 })}>
        <p className={css({ fontWeight: 'medium', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' })}>
          {name}
        </p>
        <p className={css({ color: 'text.muted', fontSize: 'xs' })}>{caption}</p>
      </div>
      {status}
      {onRemove ? (
        <button
          type="button"
          aria-label={`Remove ${name}`}
          onClick={onRemove}
          className={css({
            display: 'flex',
            p: '2',
            borderRadius: 'sm',
            color: 'text.muted',
            _hover: { color: 'text.primary', bg: 'bg.hover' },
          })}
        >
          <Trash2 size={16} />
        </button>
      ) : null}
    </header>
  )
}

function FieldRow({
  field,
  extracted,
  check,
  typed,
  onApply,
}: {
  field: SourceField
  extracted: ExtractedField
  check: FieldCheck
  typed: string
  onApply: (field: SourceField, value: string) => void
}) {
  const value = extracted.value
  if (!value) return null
  const differs = check.status === 'partial' || check.status === 'mismatch'
  const docTokens = check.status === 'partial' ? check.missingInTyped : []
  const typedTokens = check.status === 'partial' ? check.missingInDocument : []

  return (
    <div className={css({ display: 'flex', flexDirection: 'column', gap: '2' })}>
      <div className={css({ display: 'grid', gridTemplateColumns: '88px 1fr auto', gap: '3', alignItems: 'start' })}>
        <span className={css({ color: 'text.muted' })}>{fieldLabels[field]}</span>
        <span className={css({ wordBreak: 'break-word' })}>
          <Highlight text={value} tokens={docTokens} />
        </span>
        {check.status === 'match' ? (
          <span className={css({ color: 'success.fg', display: 'flex' })} aria-label="Matches">
            <CircleCheck size={16} />
          </span>
        ) : differs ? (
          <span className={css({ color: 'warning.fg', display: 'flex' })} aria-label="Differs">
            <TriangleAlert size={16} />
          </span>
        ) : (
          <span />
        )}
      </div>
      {differs ? (
        <div
          className={css({
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: '3',
            ml: '100px',
            p: '3',
            borderRadius: 'md',
            bg: 'bg.surface',
            border: '1px solid',
            borderColor: 'border.subtle',
          })}
        >
          <span className={css({ minW: 0, color: 'text.secondary', wordBreak: 'break-word' })}>
            <Highlight text={typed} tokens={typedTokens} />
          </span>
          <Button size="sm" variant="secondary" onClick={() => onApply(field, value)}>
            {useLabels[field]}
          </Button>
        </div>
      ) : null}
    </div>
  )
}

function Dates({ document }: { document: DocumentView }) {
  const { extracted } = document
  const start = formatDate(extracted.periodStart.value)
  const end = formatDate(extracted.periodEnd.value)
  const issued = formatDate(extracted.issueDate.value)
  const text = start && end ? `${start} – ${end}` : issued
  const chips: ReactNode[] = []
  for (const result of counted(document)) {
    const { freshness } = result.comparison
    const label = requirementLabels[result.requirement]
    if (freshness.status === 'fresh') {
      chips.push(
        <Badge key={result.requirement} tone="success">
          {label} · {freshness.ageDays} days
        </Badge>,
      )
    } else if (freshness.status === 'stale') {
      chips.push(
        <Badge key={result.requirement} tone="danger">
          {label} · {freshness.ageDays} days, max {freshness.maxDays}
        </Badge>,
      )
    } else if (freshness.status === 'unknown') {
      chips.push(
        <Badge key={result.requirement} tone="neutral">
          {label} · no date
        </Badge>,
      )
    }
  }
  if (!text && chips.length === 0) return null
  return (
    <div className={css({ display: 'flex', flexDirection: 'column', gap: '2' })}>
      <div className={css({ display: 'grid', gridTemplateColumns: '88px 1fr', gap: '3' })}>
        <span className={css({ color: 'text.muted' })}>{start && end ? 'Period' : 'Issued'}</span>
        <span>{text ?? '—'}</span>
      </div>
      {chips.length > 0 ? (
        <div className={css({ display: 'flex', flexWrap: 'wrap', gap: '2', ml: '100px' })}>{chips}</div>
      ) : null}
    </div>
  )
}

function Skeleton() {
  const line = css({
    h: '4',
    borderRadius: 'xs',
    backgroundImage: 'linear-gradient(90deg, rgba(255,255,255,0.04) 25%, rgba(255,255,255,0.1) 50%, rgba(255,255,255,0.04) 75%)',
    backgroundSize: '200% 100%',
    animation: 'sheen 1.6s linear infinite',
  })
  return (
    <div className={css({ display: 'flex', flexDirection: 'column', gap: '3' })}>
      <div className={line} style={{ width: '70%' }} />
      <div className={line} style={{ width: '45%' }} />
      <div className={line} style={{ width: '60%' }} />
    </div>
  )
}

function Status({ document }: { document: DocumentView }) {
  const results = counted(document)
  if (results.length === 0) return <Badge tone="danger">Not accepted</Badge>
  if (results.some(isBlocking)) return <Badge tone="warning">Needs attention</Badge>
  return <Badge tone="success">Verified</Badge>
}

export function DocumentCard({
  document,
  business,
  onApply,
  onRemove,
}: {
  document: DocumentView
  business: Business
  onApply: (field: SourceField, value: string) => void
  onRemove: () => void
}) {
  const primary = counted(document)[0]?.comparison
  const { extracted } = document

  return (
    <article className={frame}>
      <Header
        name={document.name}
        caption={`${docKindLabels[extracted.kind]} · ${formatBytes(document.size)}`}
        status={<Status document={document} />}
        onRemove={onRemove}
      />
      {primary ? (
        <div className={css({ display: 'flex', flexDirection: 'column', gap: '3.5' })}>
          <FieldRow field="legalName" extracted={extracted.legalName} check={primary.legalName} typed={business.legalName} onApply={onApply} />
          <FieldRow
            field="registrationNumber"
            extracted={document.number}
            check={primary.registrationNumber}
            typed={business.registrationNumber}
            onApply={onApply}
          />
          <FieldRow field="address" extracted={extracted.address} check={primary.address} typed={business.address} onApply={onApply} />
          <Dates document={document} />
        </div>
      ) : (
        <p className={css({ color: 'text.secondary' })}>This document type is not accepted</p>
      )}
    </article>
  )
}

export type Pending = { id: string; name: string; size: number; error?: string }

export function PendingCard({ upload, onDismiss }: { upload: Pending; onDismiss: () => void }) {
  return (
    <article className={frame}>
      <Header
        name={upload.name}
        caption={formatBytes(upload.size)}
        status={
          upload.error ? (
            <Badge tone="danger">Failed</Badge>
          ) : (
            <Badge>
              <Spinner size={12} /> Analyzing
            </Badge>
          )
        }
        onRemove={upload.error ? onDismiss : undefined}
      />
      {upload.error ? (
        <p className={css({ color: 'danger.fg' })} role="alert">
          {upload.error}
        </p>
      ) : (
        <Skeleton />
      )}
    </article>
  )
}
