import { type ReactNode, useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'

interface Props {
  title: string
  onClose: () => void
  children: ReactNode
}

/**
 * A native modal dialog, shown while mounted: it traps focus, closes on Esc,
 * on ✕ and on a click on the backdrop.
 */
export function Modal({ title, onClose, children }: Props) {
  const { t } = useTranslation()
  const ref = useRef<HTMLDialogElement>(null)

  // No close() on cleanup: its close event would arrive after StrictMode's
  // remount and shut the dialog again. Unmounting removes it anyway.
  useEffect(() => {
    if (ref.current && !ref.current.open) ref.current.showModal()
  }, [])

  return (
    <dialog
      ref={ref}
      aria-label={title}
      onClose={onClose}
      // The backdrop belongs to the dialog element itself; its content doesn't.
      onClick={(e) => e.target === e.currentTarget && onClose()}
      // Keep the page's keyboard shortcuts (Enter to continue, 1–9) out of it.
      onKeyDown={(e) => e.stopPropagation()}
      className="m-auto max-h-[90vh] w-[min(56rem,95vw)] overflow-y-auto rounded-lg p-0 backdrop:bg-black/50"
    >
      <div className="sticky top-0 flex justify-end bg-white/90 p-2">
        <button
          type="button"
          onClick={onClose}
          aria-label={t('common.close')}
          title={t('common.close')}
          className="rounded px-2 text-lg hover:bg-slate-100"
        >
          ✕
        </button>
      </div>
      <div className="px-6 pb-6">{children}</div>
    </dialog>
  )
}
