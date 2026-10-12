import { Outlet, useLocation } from '@tanstack/react-router'
import { css } from 'styled-system/css'
import { Logo } from './Logo'
import { Stepper } from './Stepper'

export function Shell() {
  const { pathname } = useLocation()
  const finished = pathname.endsWith('/done')

  return (
    <div
      className={css({
        minH: '100dvh',
        display: 'flex',
        flexDirection: 'column',
        backgroundImage: 'radial-gradient(70% 45% at 50% 0%, rgba(255, 255, 255, 0.06), transparent 70%)',
      })}
    >
      <header className={css({ display: 'flex', alignItems: 'center', h: '16', px: { base: '4', md: '6' } })}>
        <Logo />
      </header>
      <main className={css({ flex: 1, w: 'full', maxW: '560px', mx: 'auto', px: '4', pt: '6', pb: '16' })}>
        {finished ? null : <Stepper />}
        <div
          key={pathname}
          className={css({
            bg: 'bg.surface',
            border: '1px solid',
            borderColor: 'border.subtle',
            borderRadius: 'xl',
            p: { base: '5', md: '8' },
            boxShadow: 'card',
            animation: 'rise 240ms ease-out',
          })}
        >
          <Outlet />
        </div>
      </main>
    </div>
  )
}
