import { useTranslation } from 'react-i18next'
import { useConfig, useSpeechPack } from '../../shared/config/config'

/**
 * The desktop app's speech pack, offered from the header on every page until
 * it's installed: one click downloads it, then the progress shows in place.
 * Nothing on the web (speech is a service there) or once installed.
 */
export function SpeechPackOffer() {
  const { t } = useTranslation()
  const speech = useConfig().data?.speech
  const { install } = useSpeechPack()

  if (speech?.state === 'installing') {
    const percent = Math.round(speech.progress * 100)
    return (
      <span
        className="flex items-center gap-2 text-sm text-slate-600"
        title={t('settings.speech.installing')}
      >
        <span aria-hidden>🔊</span>
        <progress
          className="h-2 w-20"
          max={1}
          value={speech.progress}
          aria-label={t('settings.speech.installing')}
        />
        <span className="tabular-nums">{percent} %</span>
      </span>
    )
  }
  if (speech?.state !== 'not_installed' && speech?.state !== 'failed') return null

  const failed = speech.state === 'failed'
  return (
    <button
      type="button"
      onClick={() => install.mutate()}
      disabled={install.isPending}
      title={failed ? t('settings.speech.failed', { error: speech.error }) : t('speechOffer.hint')}
      className={`rounded-full border px-3 py-1 text-sm font-medium disabled:opacity-50 ${
        failed
          ? 'border-red-300 text-red-700 hover:bg-red-50'
          : 'border-sky-300 text-sky-700 hover:bg-sky-50'
      }`}
    >
      🔊 {failed ? t('settings.speech.retry') : t('speechOffer.enable')}
    </button>
  )
}
