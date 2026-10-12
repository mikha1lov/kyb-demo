import { defineConfig } from '@pandacss/dev'

export default defineConfig({
  presets: ['@pandacss/preset-base', '@pandacss/preset-panda'],
  preflight: true,
  include: ['./src/**/*.{ts,tsx}'],
  jsxFramework: 'react',
  outdir: 'styled-system',
  globalCss: {
    html: { colorScheme: 'dark' },
    body: {
      bg: 'bg.canvas',
      color: 'text.primary',
      fontFamily: 'sans',
      fontSize: 'sm',
      lineHeight: '1.5',
      minHeight: '100dvh',
      WebkitFontSmoothing: 'antialiased',
      MozOsxFontSmoothing: 'grayscale',
    },
    '*::selection': { bg: 'rgba(255, 255, 255, 0.18)' },
    'button, input, select, textarea': { font: 'inherit', color: 'inherit' },
    'button:not(:disabled)': { cursor: 'pointer' },
    ':focus-visible': { outline: '2px solid', outlineColor: 'focus', outlineOffset: '2px' },
  },
  theme: {
    extend: {
      tokens: {
        fonts: { sans: { value: "'Inter Variable', ui-sans-serif, system-ui, -apple-system, sans-serif" } },
        radii: {
          xs: { value: '6px' },
          sm: { value: '8px' },
          md: { value: '12px' },
          lg: { value: '16px' },
          xl: { value: '24px' },
        },
        durations: { fast: { value: '120ms' }, base: { value: '200ms' } },
      },
      semanticTokens: {
        colors: {
          bg: {
            canvas: { value: '#0a0a0b' },
            surface: { value: '#111113' },
            raised: { value: '#18181b' },
            hover: { value: '#1f1f23' },
            overlay: { value: 'rgba(0, 0, 0, 0.6)' },
          },
          border: {
            subtle: { value: 'rgba(255, 255, 255, 0.06)' },
            default: { value: 'rgba(255, 255, 255, 0.1)' },
            strong: { value: 'rgba(255, 255, 255, 0.2)' },
          },
          text: {
            primary: { value: '#f4f4f5' },
            secondary: { value: '#a1a1aa' },
            muted: { value: '#71717a' },
            inverse: { value: '#0a0a0b' },
          },
          success: {
            fg: { value: '#4ade80' },
            bg: { value: 'rgba(74, 222, 128, 0.1)' },
            border: { value: 'rgba(74, 222, 128, 0.28)' },
          },
          warning: {
            fg: { value: '#fbbf24' },
            bg: { value: 'rgba(251, 191, 36, 0.1)' },
            border: { value: 'rgba(251, 191, 36, 0.28)' },
          },
          danger: {
            fg: { value: '#f87171' },
            bg: { value: 'rgba(248, 113, 113, 0.1)' },
            border: { value: 'rgba(248, 113, 113, 0.28)' },
          },
          focus: { value: '#e4e4e7' },
        },
        shadows: {
          card: { value: '0 1px 0 rgba(255, 255, 255, 0.04) inset, 0 24px 64px -24px rgba(0, 0, 0, 0.8)' },
          pop: { value: '0 16px 48px -12px rgba(0, 0, 0, 0.7), 0 0 0 1px rgba(255, 255, 255, 0.08)' },
        },
      },
      keyframes: {
        spin: { to: { transform: 'rotate(360deg)' } },
        fadeIn: { from: { opacity: '0' }, to: { opacity: '1' } },
        rise: { from: { opacity: '0', transform: 'translateY(8px)' }, to: { opacity: '1', transform: 'translateY(0)' } },
        sheen: { from: { backgroundPosition: '200% 0' }, to: { backgroundPosition: '-200% 0' } },
      },
    },
  },
})
