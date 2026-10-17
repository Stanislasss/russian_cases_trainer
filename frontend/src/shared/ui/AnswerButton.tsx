import type { ButtonHTMLAttributes, Ref } from 'react'

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
  /** Its number key, shown in the corner (1–9). */
  shortcut?: number
  /** Something is held over it (a dragged word). */
  highlighted?: boolean
  ref?: Ref<HTMLButtonElement>
}

/** A big answer button (an ending, a case) for the games. */
export function AnswerButton({
  shortcut,
  highlighted = false,
  className = '',
  children,
  ref,
  ...props
}: Props) {
  return (
    <button
      ref={ref}
      type="button"
      className={`relative rounded-lg border-2 px-2 py-4 text-lg font-semibold transition-colors disabled:opacity-40 sm:text-xl ${
        highlighted
          ? 'border-sky-500 bg-sky-50'
          : 'border-slate-300 bg-white hover:border-slate-400'
      } ${className}`}
      {...props}
    >
      {children}
      {shortcut !== undefined && shortcut <= 9 && (
        <span className="absolute top-1 left-2 text-xs font-normal text-slate-400">{shortcut}</span>
      )}
    </button>
  )
}
