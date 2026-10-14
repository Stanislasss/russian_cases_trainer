import { useTranslation } from 'react-i18next'
import { Button } from '../../shared/ui/Button'
import type { Mode } from './gameReducer'
import { CASES, LEVEL_OPTIONS, type Settings } from './settings'

interface Props {
  settings: Settings
  onChange: (settings: Settings) => void
  onStart: () => void
  starting: boolean
  error: 'empty' | 'failed' | null
}

const MODES: Mode[] = ['practice', 'arcade']

export function SetupScreen({ settings, onChange, onStart, starting, error }: Props) {
  const { t } = useTranslation()

  const toggleLevel = (level: Settings['levels'][number]) =>
    onChange({
      ...settings,
      levels: settings.levels.includes(level)
        ? settings.levels.filter((l) => l !== level)
        : [...settings.levels, level],
    })

  return (
    <form
      className="mx-auto max-w-xl space-y-6"
      onSubmit={(e) => {
        e.preventDefault()
        onStart()
      }}
    >
      <header className="space-y-1">
        <h1 className="text-2xl font-bold">{t('game.title')}</h1>
        <p className="text-slate-600">{t('game.subtitle')}</p>
      </header>

      <fieldset className="space-y-2">
        <legend className="font-semibold">{t('game.levels')}</legend>
        {LEVEL_OPTIONS.map(({ id, level }) => (
          <label key={id} className={`flex items-center gap-2 ${level ? '' : 'text-slate-400'}`}>
            <input
              type="checkbox"
              disabled={!level}
              checked={level ? settings.levels.includes(level) : false}
              onChange={() => level && toggleLevel(level)}
            />
            {t(`game.level.${id}`)}
            {!level && <span className="text-xs">({t('game.comingSoon')})</span>}
          </label>
        ))}
      </fieldset>

      <fieldset>
        <legend className="mb-2 font-semibold">{t('game.case')}</legend>
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
          {CASES.map((c) => (
            <label
              key={c}
              className="flex cursor-pointer items-center gap-2 rounded border border-slate-300 px-3 py-2 has-checked:border-sky-600 has-checked:bg-sky-50"
            >
              <input
                type="radio"
                name="case"
                checked={settings.case === c}
                onChange={() => onChange({ ...settings, case: c })}
              />
              {t(`cases.${c}`)}
            </label>
          ))}
        </div>
      </fieldset>

      <fieldset>
        <legend className="mb-2 font-semibold">{t('game.mode')}</legend>
        <div className="grid gap-2 sm:grid-cols-2">
          {MODES.map((m) => (
            <label
              key={m}
              className="flex cursor-pointer gap-2 rounded border border-slate-300 px-3 py-2 has-checked:border-sky-600 has-checked:bg-sky-50"
            >
              <input
                type="radio"
                name="mode"
                checked={settings.mode === m}
                onChange={() => onChange({ ...settings, mode: m })}
              />
              <span>
                <span className="block font-medium">{t(`game.modes.${m}`)}</span>
                <span className="block text-sm text-slate-600">{t(`game.modes.${m}Hint`)}</span>
              </span>
            </label>
          ))}
        </div>
      </fieldset>

      {error && (
        <p role="alert" className="text-sm text-red-600">
          {t(error === 'empty' ? 'game.empty' : 'game.failed')}
        </p>
      )}
      <Button type="submit" disabled={settings.levels.length === 0 || starting} className="w-full">
        {starting ? t('common.loading') : t('game.start')}
      </Button>
    </form>
  )
}
