import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import { css } from 'styled-system/css'
import { Badge } from '#/components/ui/Badge'
import { Button } from '#/components/ui/Button'
import { Field, TextInput } from '#/components/ui/Field'
import { detailsSchema, type Details } from '#/lib/schema'
import type { Suggestion, Suggestions } from '#/lib/types'
import { StepLayout } from './StepLayout'

function messageOf(errors: unknown[]) {
  const first = errors.find(Boolean) as { message?: string } | string | undefined
  return typeof first === 'string' ? first : first?.message
}

type Validated = {
  form: { state: { submissionAttempts: number } }
  state: { meta: { isBlurred: boolean; errors: unknown[] } }
}

function showError(field: Validated) {
  const visible = field.state.meta.isBlurred || field.form.state.submissionAttempts > 0
  return visible ? messageOf(field.state.meta.errors) : undefined
}

function fromDocument(value: string, suggestion: Suggestion | undefined) {
  return suggestion && suggestion.value === value ? <Badge>From document</Badge> : null
}

export function DetailsForm({
  initial,
  suggestions,
  onBack,
  onSubmit,
}: {
  initial: Details
  suggestions: Suggestions
  onBack: () => void
  onSubmit: (details: Details) => Promise<void>
}) {
  const [failure, setFailure] = useState<string | null>(null)

  const form = useForm({
    defaultValues: initial,
    validators: { onChange: detailsSchema, onSubmit: detailsSchema },
    onSubmit: async ({ value }) => {
      setFailure(null)
      try {
        await onSubmit(detailsSchema.parse(value))
      } catch (error) {
        setFailure(error instanceof Error ? error.message : 'Something went wrong')
      }
    },
  })

  return (
    <form
      noValidate
      onSubmit={(event) => {
        event.preventDefault()
        void form.handleSubmit()
      }}
    >
      <StepLayout
        title="Business details"
        footer={
          <>
            <Button variant="secondary" size="lg" onClick={onBack}>
              Back
            </Button>
            <form.Subscribe selector={(state) => state.isSubmitting}>
              {(submitting) => (
                <Button type="submit" size="lg" className={css({ flex: 1 })} loading={submitting}>
                  Continue
                </Button>
              )}
            </form.Subscribe>
          </>
        }
      >
        <form.Field name="legalName">
          {(field) => (
            <Field
              label="Legal name"
              htmlFor="legalName"
              error={showError(field)}
              tag={fromDocument(field.state.value, suggestions.legalName)}
            >
              <TextInput
                id="legalName"
                autoComplete="organization"
                value={field.state.value}
                invalid={showError(field) !== undefined}
                onChange={(event) => field.handleChange(event.target.value)}
                onBlur={field.handleBlur}
              />
            </Field>
          )}
        </form.Field>
        <form.Field name="registrationNumber">
          {(field) => (
            <Field
              label="Registration number"
              htmlFor="registrationNumber"
              error={showError(field)}
              tag={fromDocument(field.state.value, suggestions.registrationNumber)}
            >
              <TextInput
                id="registrationNumber"
                inputMode="numeric"
                autoComplete="off"
                value={field.state.value}
                invalid={showError(field) !== undefined}
                onChange={(event) => field.handleChange(event.target.value)}
                onBlur={field.handleBlur}
              />
            </Field>
          )}
        </form.Field>
        <form.Field name="address">
          {(field) => (
            <Field
              label="Registered address"
              htmlFor="address"
              error={showError(field)}
              tag={fromDocument(field.state.value, suggestions.address)}
            >
              <TextInput
                id="address"
                autoComplete="street-address"
                value={field.state.value}
                invalid={showError(field) !== undefined}
                onChange={(event) => field.handleChange(event.target.value)}
                onBlur={field.handleBlur}
              />
            </Field>
          )}
        </form.Field>
        {failure ? (
          <p role="alert" className={css({ color: 'danger.fg', fontSize: 'xs' })}>
            {failure}
          </p>
        ) : null}
      </StepLayout>
    </form>
  )
}
