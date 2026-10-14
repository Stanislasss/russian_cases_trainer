import type { ButtonHTMLAttributes } from 'react'

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: 'primary' | 'ghost'
}

const variants = {
  primary: 'bg-sky-600 text-white hover:bg-sky-700',
  ghost: 'text-slate-700 hover:bg-slate-100',
}

export function Button({ variant = 'primary', className = '', ...props }: Props) {
  return (
    <button
      className={`rounded px-4 py-2 text-sm font-medium disabled:opacity-50 ${variants[variant]} ${className}`}
      {...props}
    />
  )
}
