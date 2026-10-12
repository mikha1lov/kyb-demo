import { Combobox, useListCollection } from '@ark-ui/react/combobox'
import { useFilter } from '@ark-ui/react/locale'
import { Portal } from '@ark-ui/react/portal'
import { Check, ChevronDown } from 'lucide-react'
import { css } from 'styled-system/css'
import { countries, type Country } from '#/lib/countries'
import { control } from './Field'

export function CountrySelect({
  id,
  value,
  invalid,
  onChange,
  onBlur,
}: {
  id: string
  value: string
  invalid?: boolean
  onChange: (code: string) => void
  onBlur?: () => void
}) {
  const { contains } = useFilter({ sensitivity: 'base' })
  const { collection, filter } = useListCollection<Country>({
    initialItems: countries,
    filter: contains,
    itemToString: (c) => c.name,
    itemToValue: (c) => c.code,
  })

  return (
    <Combobox.Root
      collection={collection}
      value={value ? [value] : []}
      ids={{ input: id }}
      openOnClick
      positioning={{ sameWidth: true, gutter: 6 }}
      onValueChange={(details) => onChange(details.value[0] ?? '')}
      onInputValueChange={(details) => filter(details.inputValue)}
      className={css({ w: 'full' })}
    >
      <Combobox.Control className={css({ position: 'relative' })}>
        <Combobox.Input
          placeholder="Select a country"
          aria-invalid={invalid || undefined}
          onBlur={onBlur}
          className={`${control} ${css({ pr: '10' })}`}
        />
        <Combobox.Trigger
          aria-label="Open countries"
          className={css({
            position: 'absolute',
            right: '3',
            top: '50%',
            transform: 'translateY(-50%)',
            color: 'text.muted',
            display: 'flex',
          })}
        >
          <ChevronDown size={16} />
        </Combobox.Trigger>
      </Combobox.Control>
      <Portal>
        <Combobox.Positioner className={css({ zIndex: 'popover' })}>
          <Combobox.Content
            className={css({
              maxH: '72',
              overflowY: 'auto',
              p: '1.5',
              bg: 'bg.raised',
              borderRadius: 'md',
              boxShadow: 'pop',
              animation: 'fadeIn 120ms ease-out',
              _focus: { outline: 'none' },
            })}
          >
            <Combobox.Empty className={css({ px: '3', py: '2.5', color: 'text.muted' })}>
              No countries found
            </Combobox.Empty>
            {collection.items.map((country) => (
              <Combobox.Item
                key={country.code}
                item={country}
                className={css({
                  display: 'flex',
                  alignItems: 'center',
                  gap: '3',
                  px: '3',
                  py: '2',
                  borderRadius: 'sm',
                  cursor: 'pointer',
                  _highlighted: { bg: 'bg.hover' },
                })}
              >
                <span aria-hidden>{country.flag}</span>
                <Combobox.ItemText className={css({ flex: 1 })}>{country.name}</Combobox.ItemText>
                <Combobox.ItemIndicator className={css({ display: 'flex', color: 'text.secondary' })}>
                  <Check size={16} />
                </Combobox.ItemIndicator>
              </Combobox.Item>
            ))}
          </Combobox.Content>
        </Combobox.Positioner>
      </Portal>
    </Combobox.Root>
  )
}
