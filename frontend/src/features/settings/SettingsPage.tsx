import { useTranslation } from 'react-i18next'
import { useConfig, useSpeechPack } from '../../shared/config/config'
import { Button } from '../../shared/ui/Button'
import { PracticeSettings } from './PracticeSettings'

/** The desktop app's settings: how words are picked, and the downloadable speech pack. */
export function SettingsPage() {
  const { t } = useTranslation()
  const config = useConfig()
  const { install, remove } = useSpeechPack()
  const speech = config.data?.speech
  const busy = install.isPending || remove.isPending

  return (
    <section className="max-w-xl space-y-6">
      <h1 className="text-2xl font-bold">{t('settings.title')}</h1>

      <PracticeSettings />

      <div className="space-y-3 rounded-lg border border-slate-200 bg-white p-4">
        <h2 className="text-lg font-semibold">{t('settings.speech.title')}</h2>
        <p className="text-sm text-slate-600">{t('settings.speech.about')}</p>

        {speech?.state === 'service' && <p>{t('settings.speech.service')}</p>}

        {(speech?.state === 'not_installed' || speech?.state === 'failed') && (
          <>
            {speech.state === 'failed' && (
              <p role="alert" className="text-sm text-red-700">
                {t('settings.speech.failed', { error: speech.error })}
              </p>
            )}
            <Button onClick={() => install.mutate()} disabled={busy}>
              {t(speech.state === 'failed' ? 'settings.speech.retry' : 'settings.speech.install')}
            </Button>
          </>
        )}

        {speech?.state === 'installing' && (
          <div className="space-y-1">
            <p className="text-sm">{t('settings.speech.installing')}</p>
            <progress
              className="w-full"
              max={1}
              value={speech.progress}
              aria-label={t('settings.speech.installing')}
            />
            <p className="text-right text-xs text-slate-500">
              {Math.round(speech.progress * 100)} %
            </p>
          </div>
        )}

        {speech?.state === 'ready' && (
          <div className="flex items-center justify-between gap-3">
            <p className="text-green-700">✓ {t('settings.speech.ready')}</p>
            <Button variant="ghost" onClick={() => remove.mutate()} disabled={busy}>
              {t('settings.speech.remove')}
            </Button>
          </div>
        )}

        <p className="text-xs text-slate-500">
          {t('settings.speech.licence')}{' '}
          <a className="underline" href="https://github.com/rhasspy/piper">
            Piper
          </a>
        </p>
      </div>
    </section>
  )
}
