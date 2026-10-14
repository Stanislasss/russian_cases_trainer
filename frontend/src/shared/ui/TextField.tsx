import { useId, type ComponentProps } from 'react'

// ComponentProps includes `ref`, so react-hook-form's register() can be spread in.
type Props = ComponentProps<'input'> & {
  label: string
  error?: string
}

export function TextField({ label, error, ...input }: Props) {
  const id = useId()
  const errorId = `${id}-error`

  return (
    <div className="space-y-1">
      <label htmlFor={id} className="block text-sm font-medium">
        {label}
      </label>
      <input
        id={id}
        aria-invalid={error ? true : undefined}
        aria-describedby={error ? errorId : undefined}
        className="w-full rounded border border-slate-300 px-3 py-2 focus:border-sky-500 focus:outline-none aria-invalid:border-red-500"
        {...input}
      />
      {error && (
        <p id={errorId} className="text-sm text-red-600">
          {error}
        </p>
      )}
    </div>
  )
}
