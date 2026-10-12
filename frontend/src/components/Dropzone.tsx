import { CloudUpload } from 'lucide-react'
import { useRef, useState } from 'react'
import { css } from 'styled-system/css'

export const acceptedTypes = ['application/pdf', 'image/png', 'image/jpeg', 'image/webp']
export const maxFileBytes = 10 * 1024 * 1024

export function Dropzone({ onFiles }: { onFiles: (files: File[]) => void }) {
  const input = useRef<HTMLInputElement>(null)
  const [over, setOver] = useState(false)

  const pick = (list: FileList | null) => {
    if (list?.length) onFiles(Array.from(list))
  }

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={() => input.current?.click()}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault()
          input.current?.click()
        }
      }}
      onDragOver={(event) => {
        event.preventDefault()
        setOver(true)
      }}
      onDragLeave={() => setOver(false)}
      onDrop={(event) => {
        event.preventDefault()
        setOver(false)
        pick(event.dataTransfer.files)
      }}
      className={css({
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '3',
        py: '8',
        px: '4',
        borderRadius: 'lg',
        border: '1px dashed',
        borderColor: over ? 'text.secondary' : 'border.strong',
        bg: over ? 'bg.raised' : 'transparent',
        color: 'text.secondary',
        textAlign: 'center',
        transition: 'background-color 120ms, border-color 120ms',
        _hover: { bg: 'bg.raised', borderColor: 'text.muted' },
      })}
    >
      <input
        ref={input}
        type="file"
        multiple
        hidden
        accept={acceptedTypes.join(',')}
        onChange={(event) => {
          pick(event.target.files)
          event.target.value = ''
        }}
      />
      <CloudUpload size={22} />
      <span className={css({ fontWeight: 'medium', color: 'text.primary' })}>Drop files or browse</span>
    </div>
  )
}
