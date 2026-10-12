import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import { css } from 'styled-system/css'
import { Button } from '#/components/ui/Button'
import { CountrySelect } from '#/components/ui/CountrySelect'
import { Field, TextInput } from '#/components/ui/Field'
import { Segmented } from '#/components/ui/Segmented'
import { businessSchema, entityTypeLabels, entityTypes, type Business } from '#/lib/schema'
import { StepLayout } from './StepLayout'

const entityOptions = entityTypes.map((value) => ({ value, label: entityTypeLabels[value] }))

const blank: Business = { country: '', entityType: 'sole_proprietor', legalName: '', registrationNumber: '', address: '' }

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

export function BusinessForm({
  initial,
  onSubmit,
}: {
  initial: Business | null
  onSubmit: (business: Business) => Promise<void>
}) {
  const [failure, setFailure] = useState<string | null>(null)

  const form = useForm({
    defaultValues: initial ?? blank,
    validators: { onChange: businessSchema, onSubmit: businessSchema },
    onSubmit: async ({ value }) => {
      setFailure(null)
      try {
        await onSubmit(businessSchema.parse(value))
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
          <form.Subscribe selector={(state) => state.isSubmitting}>
            {(submitting) => (
              <Button type="submit" size="lg" className={css({ flex: 1 })} loading={submitting}>
                Continue
              </Button>
            )}
          </form.Subscribe>
        }
      >
        <form.Field name="country">
          {(field) => (
            <Field label="Country" htmlFor="country" error={showError(field)}>
              <CountrySelect
                id="country"
                value={field.state.value}
                invalid={showError(field) !== undefined}
                onChange={field.handleChange}
                onBlur={field.handleBlur}
              />
            </Field>
          )}
        </form.Field>
        <form.Field name="entityType">
          {(field) => (
            <Field label="Entity type">
              <Segmented name="entityType" value={field.state.value} options={entityOptions} onChange={field.handleChange} />
            </Field>
          )}
        </form.Field>
        <form.Field name="legalName">
          {(field) => (
            <Field label="Legal name" htmlFor="legalName" error={showError(field)}>
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
            <Field label="Registration number" htmlFor="registrationNumber" error={showError(field)}>
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
            <Field label="Registered address" htmlFor="address" error={showError(field)}>
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
