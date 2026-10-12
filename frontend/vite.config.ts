import { defineConfig } from 'vite'
import { tanstackStart } from '@tanstack/react-start/plugin/vite'
import viteReact from '@vitejs/plugin-react'

export default defineConfig({
  resolve: { tsconfigPaths: true },
  server: {
    proxy: {
      '/api': { target: process.env.API_URL ?? 'http://127.0.0.1:8787', changeOrigin: true },
    },
  },
  plugins: [tanstackStart(), viteReact()],
})
