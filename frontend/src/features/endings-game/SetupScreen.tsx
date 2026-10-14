import { useTranslation } from 'react-i18next'
import { Button } from '../../shared/ui/Button'
import type { Mode } from './gameReducer'
import {
  CASES,
  LEVEL_GROUPS,
  NEXT_WORD_DELAYS,
  type Settings,
  SOLO_LEVELS,
  toggleLevel,
} from './settings'

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

  const toggle = (level: Settings['levels'][number]) =>
    onChange({ ...settings, levels: toggleLevel(settings.levels, level) })

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

      <fieldset className="space-y-3">
        <legend className="font-semibold">{t('game.levels')}</legend>
        {LEVEL_GROUPS.map((group) => (
          // A fieldset per group, so "Intermediate" is announced as nouns or adjectives.
          <fieldset key={group.id} className="space-y-1">
            <legend className="text-sm font-medium text-slate-500">
              {t(`game.levelGroups.${group.id}`)}
            </legend>
            {group.levels.map((level) => (
              <label key={level} className="flex items-baseline gap-2">
                <input
                  type="checkbox"
                  checked={settings.levels.includes(level)}
                  onChange={() => toggle(level)}
                />
                <span>
                  {t(`game.level.${level}`)}
                  {SOLO_LEVELS.includes(level) && (
                    <span className="block text-xs text-slate-500">{t('game.soloHint')}</span>
                  )}
                </span>
              </label>
            ))}
          </fieldset>
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

      {settings.mode === 'practice' && (
        <label className="flex items-center justify-between gap-3">
          <span className="font-semibold">{t('game.nextWordDelay')}</span>
          <select
            className="rounded border border-slate-300 bg-white px-2 py-1"
            value={settings.nextWordDelay}
            onChange={(e) => onChange({ ...settings, nextWordDelay: Number(e.target.value) })}
          >
            {NEXT_WORD_DELAYS.map((ms) => (
              <option key={ms} value={ms}>
                {t('game.seconds', { count: ms / 1000 })}
              </option>
            ))}
          </select>
        </label>
      )}

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
