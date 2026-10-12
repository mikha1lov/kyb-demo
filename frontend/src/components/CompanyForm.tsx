import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import { css } from 'styled-system/css'
import { Button } from '#/components/ui/Button'
import { CountrySelect } from '#/components/ui/CountrySelect'
import { Field } from '#/components/ui/Field'
import { Segmented } from '#/components/ui/Segmented'
import { companySchema, entityTypeLabels, entityTypes, type Company } from '#/lib/schema'
import { StepLayout } from './StepLayout'

const entityOptions = entityTypes.map((value) => ({ value, label: entityTypeLabels[value] }))

const blank: Company = { country: '', entityType: 'sole_proprietor' }

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

export function CompanyForm({
  initial,
  onSubmit,
}: {
  initial: Company | null
  onSubmit: (company: Company) => Promise<void>
}) {
  const [failure, setFailure] = useState<string | null>(null)

  const form = useForm({
    defaultValues: initial ?? blank,
    validators: { onChange: companySchema, onSubmit: companySchema },
    onSubmit: async ({ value }) => {
      setFailure(null)
      try {
        await onSubmit(companySchema.parse(value))
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
        title="Company"
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
        {failure ? (
          <p role="alert" className={css({ color: 'danger.fg', fontSize: 'xs' })}>
            {failure}
          </p>
        ) : null}
      </StepLayout>
    </form>
  )
}
