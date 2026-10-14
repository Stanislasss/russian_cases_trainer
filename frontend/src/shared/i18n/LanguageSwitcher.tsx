import { useTranslation } from 'react-i18next'
import { languages } from '.'

export function LanguageSwitcher() {
  const { i18n, t } = useTranslation()

  return (
    <select
      aria-label={t('app.language')}
      className="rounded border border-slate-300 bg-white px-2 py-1 text-sm"
      value={i18n.resolvedLanguage}
      onChange={(e) => void i18n.changeLanguage(e.target.value)}
    >
      {languages.map((lng) => (
        <option key={lng} value={lng}>
          {t(`languages.${lng}`)}
        </option>
      ))}
    </select>
  )
}
