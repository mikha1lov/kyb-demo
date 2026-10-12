import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
import { useSelector } from '@tanstack/react-store'
import { useState } from 'react'
import { css } from 'styled-system/css'
import { Checklist } from '#/components/Checklist'
import { DocumentCard } from '#/components/DocumentCard'
import { acceptedTypes, Dropzone, maxFileBytes } from '#/components/Dropzone'
import { StepLayout } from '#/components/StepLayout'
import { Button } from '#/components/ui/Button'
import { addDocument, draft } from '#/lib/draft'
import { missing } from '#/lib/requirements'

export const Route = createFileRoute('/onboarding/documents')({
  beforeLoad: () => {
    if (!draft.get().business) throw redirect({ to: '/onboarding/business' })
  },
  component: DocumentsStep,
})

function DocumentsStep() {
  const navigate = useNavigate()
  const business = useSelector(draft, (s) => s.business)
  const documents = useSelector(draft, (s) => s.documents)
  const [rejected, setRejected] = useState<string[]>([])

  if (!business) return null

  const analyzing = documents.some((d) => d.status === 'analyzing')
  const complete = missing(documents).length === 0 && !analyzing

  const handleFiles = (files: File[]) => {
    const problems: string[] = []
    for (const file of files) {
      if (!acceptedTypes.includes(file.type)) problems.push(`${file.name} is not a PDF or image`)
      else if (file.size > maxFileBytes) problems.push(`${file.name} is larger than 10 MB`)
      else void addDocument(file)
    }
    setRejected(problems)
  }

  return (
    <StepLayout
      title="Documents"
      footer={
        <>
          <Button variant="secondary" size="lg" onClick={() => navigate({ to: '/onboarding/business' })}>
            Back
          </Button>
          <Button
            size="lg"
            className={css({ flex: 1 })}
            disabled={!complete}
            onClick={() => navigate({ to: '/onboarding/review' })}
          >
            Continue
          </Button>
        </>
      }
    >
      <Checklist documents={documents} />
      <Dropzone onFiles={handleFiles} />
      {rejected.map((message) => (
        <p key={message} role="alert" className={css({ color: 'danger.fg', fontSize: 'xs' })}>
          {message}
        </p>
      ))}
      {documents.map((document) => (
        <DocumentCard key={document.id} document={document} business={business} />
      ))}
    </StepLayout>
  )
}
