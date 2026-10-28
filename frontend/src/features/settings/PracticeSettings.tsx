import { useTranslation } from 'react-i18next'
import { usePreferences, useSavePreferences } from '../../shared/preferences/preferences'

/** How words are picked; on the desktop settings page and the web account page. */
export function PracticeSettings() {
  const { t } = useTranslation()
  const preferences = usePreferences()
  const save = useSavePreferences()
  const current = preferences.data

  return (
    <div className="space-y-3 rounded-lg border border-slate-200 bg-white p-4">
      <h2 className="text-lg font-semibold">{t('settings.practice.title')}</h2>
      <label className="flex items-baseline gap-2">
        <input
          type="checkbox"
          checked={current?.fewer_known_words ?? true}
          disabled={!current}
          onChange={(e) => save.mutate({ ...current, fewer_known_words: e.target.checked })}
        />
        <span>
          {t('settings.practice.fewerKnownWords')}
          <span className="block text-sm text-slate-600">
            {t('settings.practice.fewerKnownWordsAbout')}
          </span>
        </span>
      </label>
      {(preferences.isError || save.isError) && (
        <p role="alert" className="text-sm text-red-700">
          {t('auth.errors.internal')}
        </p>
      )}
    </div>
  )
}
