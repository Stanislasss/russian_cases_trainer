import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link, useNavigate, useSearchParams } from 'react-router'
import { z } from 'zod'
import { ApiError } from '../../shared/api/client'
import { Button } from '../../shared/ui/Button'
import { TextField } from '../../shared/ui/TextField'
import { safeNext } from './safeNext'
import { useLogin } from '../../shared/session/session'

const schema = z.object({
  email: z.email({ error: 'invalid_email' }),
  password: z.string().min(1, { error: 'required' }),
})
type Values = z.infer<typeof schema>

export function LoginPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [params] = useSearchParams()
  const login = useLogin()
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<Values>({ resolver: zodResolver(schema) })

  const onSubmit = handleSubmit((values) =>
    login.mutate(values, { onSuccess: () => navigate(safeNext(params.get('next'))) }),
  )
  const serverError = login.error instanceof ApiError ? login.error.code : login.error && 'internal'

  return (
    <form onSubmit={onSubmit} noValidate className="mx-auto max-w-sm space-y-4">
      <h1 className="text-2xl font-bold">{t('auth.login.title')}</h1>
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
        autoComplete="current-password"
        error={errors.password && t(`auth.errors.${errors.password.message}`)}
        {...register('password')}
      />
      {serverError && (
        <p role="alert" className="text-sm text-red-600">
          {t(`auth.errors.${serverError}`)}
        </p>
      )}
      <Button type="submit" disabled={login.isPending} className="w-full">
        {t('auth.login.submit')}
      </Button>
      <p className="text-center text-sm">
        {t('auth.login.noAccount')}{' '}
        <Link to="/register" className="text-sky-700 underline">
          {t('auth.register.title')}
        </Link>
      </p>
    </form>
  )
}
