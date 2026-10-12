import { useForm } from '@tanstack/react-form'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useSelector } from '@tanstack/react-store'
import { css } from 'styled-system/css'
import { Button } from '#/components/ui/Button'
import { CountrySelect } from '#/components/ui/CountrySelect'
import { Field, TextInput } from '#/components/ui/Field'
import { Segmented } from '#/components/ui/Segmented'
import { StepLayout } from '#/components/StepLayout'
import { draft, saveBusiness } from '#/lib/draft'
import { businessSchema, entityTypeLabels, entityTypes, type Business } from '#/lib/schema'

export const Route = createFileRoute('/onboarding/business')({ component: BusinessStep })

const entityOptions = entityTypes.map((value) => ({ value, label: entityTypeLabels[value] }))

const blank: Business = { country: '', entityType: 'sole_proprietor', legalName: '', registrationNumber: '', address: '' }

function messageOf(errors: unknown[]) {
  const first = errors.find(Boolean) as { message?: string } | string | undefined
  return typeof first === 'string' ? first : first?.message
}

type Touchable = { state: { meta: { isTouched: boolean; errors: unknown[] } } }

function showError(field: Touchable) {
  return field.state.meta.isTouched ? messageOf(field.state.meta.errors) : undefined
}

function BusinessStep() {
  const navigate = useNavigate()
  const saved = useSelector(draft, (s) => s.business)

  const form = useForm({
    defaultValues: saved ?? blank,
    validators: { onBlur: businessSchema, onSubmit: businessSchema },
    onSubmit: async ({ value }) => {
      saveBusiness(businessSchema.parse(value))
      await navigate({ to: '/onboarding/documents' })
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
          <Button type="submit" size="lg" className={css({ flex: 1 })}>
            Continue
          </Button>
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
      </StepLayout>
    </form>
  )
}
