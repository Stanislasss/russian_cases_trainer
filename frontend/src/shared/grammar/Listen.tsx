import { useTranslation } from 'react-i18next'
import { speakable } from './format'

/** A small 🔊 button for a Russian text; nothing without `speak` (speech off). */
export function Listen({ text, speak }: { text: string; speak?: (text: string) => void }) {
  const { t } = useTranslation()
  if (!speak) return null
  return (
    <button
      type="button"
      onClick={() => speak(speakable(text))}
      aria-label={`${t('game.listen')}: ${text}`}
      title={t('game.listen')}
      className="ml-1 rounded px-1 align-middle hover:bg-slate-200"
    >
      🔊
    </button>
  )
}
