import { useRouter } from '@tanstack/react-router'
import { css } from 'styled-system/css'
import { Button } from '#/components/ui/Button'

export function ErrorPanel() {
  const router = useRouter()
  return (
    <div className={css({ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '5', py: '8' })}>
      <h1 className={css({ fontSize: 'xl', fontWeight: 'semibold' })}>Something went wrong</h1>
      <Button variant="secondary" onClick={() => void router.invalidate()}>
        Try again
      </Button>
    </div>
  )
}
