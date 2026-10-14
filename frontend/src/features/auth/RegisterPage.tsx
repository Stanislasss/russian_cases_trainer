import { zodResolver } from '@hookform/resolvers/zod'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link, useNavigate } from 'react-router'
import { z } from 'zod'
import { ApiError, type ErrorCode } from '../../shared/api/client'
import { languages, type Language } from '../../shared/i18n'
import { Button } from '../../shared/ui/Button'
import { TextField } from '../../shared/ui/TextField'
import { useRegister } from './session'

// Mirrors the backend's rules (domain::user) so most mistakes are caught before submitting.
const schema = z.object({
  display_name: z
    .string()
    .trim()
    .min(1, { error: 'required' })
    .max(50, { error: 'invalid_display_name' }),
  email: z.email({ error: 'invalid_email' }),
  password: z
    .string()
    .min(8, { error: 'invalid_password' })
    .max(128, { error: 'invalid_password' }),
})
type Values = z.infer<typeof schema>

export function RegisterPage() {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const registration = useRegister()
  const {
    register,
    handleSubmit,
    setError,
    formState: { errors },
  } = useForm<Values>({ resolver: zodResolver(schema) })

  const [formError, setFormError] = useState<ErrorCode | null>(null)

  const uiLocale = languages.includes(i18n.resolvedLanguage as Language)
    ? (i18n.resolvedLanguage as Language)
    : 'en'
  const onSubmit = handleSubmit((values) => {
    setFormError(null)
    registration.mutate(
      { ...values, ui_locale: uiLocale },
      {
        onSuccess: () => navigate('/'),
        // Server-side validation errors go next to their field, anything else above the button.
        onError: (error) => {
          const code = error instanceof ApiError ? error.code : 'internal'
          const field = fieldOf[code]
          if (field) setError(field, { message: code })
          else setFormError(code)
        },
      },
    )
  })

  return (
    <form onSubmit={onSubmit} noValidate className="mx-auto max-w-sm space-y-4">
      <h1 className="text-2xl font-bold">{t('auth.register.title')}</h1>
      <TextField
        label={t('auth.displayName')}
        autoComplete="nickname"
        error={errors.display_name && t(`auth.errors.${errors.display_name.message}`)}
        {...register('display_name')}
      />
      <TextField
        label={t('auth.email')}
        type="email"
        autoComplete="email"
        error={errors.email && t(`auth.errors.${errors.email.message}`)}
        {...register('email')}
      />
      <TextField
        label={t('auth.password')}
        type="password"
        autoComplete="new-password"
        error={errors.password && t(`auth.errors.${errors.password.message}`)}
        {...register('password')}
      />
      {formError && (
        <p role="alert" className="text-sm text-red-600">
          {t(`auth.errors.${formError}`)}
        </p>
      )}
      <Button type="submit" disabled={registration.isPending} className="w-full">
        {t('auth.register.submit')}
      </Button>
      <p className="text-center text-sm">
        {t('auth.register.haveAccount')}{' '}
        <Link to="/login" className="text-sky-700 underline">
          {t('auth.login.title')}
        </Link>
      </p>
    </form>
  )
}

const fieldOf: Partial<Record<ErrorCode, keyof Values>> = {
  email_taken: 'email',
  invalid_email: 'email',
  invalid_password: 'password',
  invalid_display_name: 'display_name',
}
