import { Navigate, createFileRoute, useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { css } from 'styled-system/css'
import { Checklist } from '#/components/Checklist'
import { DocumentCard, PendingCard, type Pending } from '#/components/DocumentCard'
import { acceptedTypes, Dropzone, maxFileBytes } from '#/components/Dropzone'
import { StepLayout } from '#/components/StepLayout'
import { Button } from '#/components/ui/Button'
import { deleteDocument, saveBusiness, uploadDocument } from '#/lib/api'
import { messageOf } from '#/lib/errors'
import type { SourceField } from '#/lib/types'
import { useApplication, useReload } from '#/lib/useApplication'

export const Route = createFileRoute('/onboarding/$id/documents')({ component: DocumentsStep })

function DocumentsStep() {
  const application = useApplication()
  const reload = useReload()
  const navigate = useNavigate()
  const [pending, setPending] = useState<Pending[]>([])
  const [rejected, setRejected] = useState<string[]>([])
  const [failure, setFailure] = useState<string | null>(null)
  const { business } = application
  const params = { id: application.id }

  if (application.status === 'submitted') return <Navigate to="/onboarding/$id/done" params={params} replace />
  if (!business) return <Navigate to="/onboarding/$id/business" params={params} replace />

  const uploading = pending.some((p) => !p.error)

  const upload = (file: File) => {
    const id = crypto.randomUUID()
    setPending((list) => [...list, { id, name: file.name, size: file.size }])
    uploadDocument(application.id, file)
      .then(() => reload())
      .then(() => setPending((list) => list.filter((p) => p.id !== id)))
      .catch((error) => setPending((list) => list.map((p) => (p.id === id ? { ...p, error: messageOf(error) } : p))))
  }

  const handleFiles = (files: File[]) => {
    const problems: string[] = []
    for (const file of files) {
      if (!acceptedTypes.includes(file.type)) problems.push(`${file.name} is not a PDF or image`)
      else if (file.size > maxFileBytes) problems.push(`${file.name} is larger than 10 MB`)
      else upload(file)
    }
    setRejected(problems)
  }

  const run = async (action: () => Promise<unknown>) => {
    setFailure(null)
    try {
      await action()
      await reload()
    } catch (error) {
      setFailure(messageOf(error))
    }
  }

  const apply = (field: SourceField, value: string) =>
    run(() => saveBusiness(application.id, { ...business, [field]: value }, { ...application.sources, [field]: 'document' }))

  const remove = (documentId: string) => run(() => deleteDocument(application.id, documentId))

  return (
    <StepLayout
      title="Documents"
      footer={
        <>
          <Button variant="secondary" size="lg" onClick={() => navigate({ to: '/onboarding/$id/business', params })}>
            Back
          </Button>
          <Button
            size="lg"
            className={css({ flex: 1 })}
            disabled={!application.complete || uploading}
            onClick={() => navigate({ to: '/onboarding/$id/review', params })}
          >
            Continue
          </Button>
        </>
      }
    >
      <Checklist application={application} />
      <Dropzone onFiles={handleFiles} />
      {[...rejected, ...(failure ? [failure] : [])].map((message) => (
        <p key={message} role="alert" className={css({ color: 'danger.fg', fontSize: 'xs' })}>
          {message}
        </p>
      ))}
      {application.documents.map((document) => (
        <DocumentCard
          key={document.id}
          document={document}
          business={business}
          onApply={apply}
          onRemove={() => remove(document.id)}
        />
      ))}
      {pending.map((upload) => (
        <PendingCard
          key={upload.id}
          upload={upload}
          onDismiss={() => setPending((list) => list.filter((p) => p.id !== upload.id))}
        />
      ))}
    </StepLayout>
  )
}
