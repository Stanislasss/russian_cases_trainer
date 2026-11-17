import { useTranslation } from 'react-i18next'
import { Button } from '../../shared/ui/Button'
import { CASES } from '../endings-game/settings'
import type { Level } from './api'
import {
  NOUN_LEVELS,
  NUMBER_CHOICES,
  playableCases,
  ROUND_LENGTHS,
  type Settings,
  toggleCase,
} from './settings'

interface Props {
  settings: Settings
  onChange: (settings: Settings) => void
  onStart: () => void
  starting: boolean
  error: 'empty' | 'failed' | null
}

/** Choices made before a round: levels, cases, number, length and help. */
export function SetupForm({ settings, onChange, onStart, starting, error }: Props) {
  const { t } = useTranslation()
  const singular = settings.number === 'singular'
  const noCase = playableCases(settings).length === 0
  const choice =
    'flex cursor-pointer items-center gap-2 rounded border border-slate-300 px-3 py-2 has-checked:border-sky-600 has-checked:bg-sky-50 has-disabled:cursor-not-allowed has-disabled:opacity-50'

  return (
    <form
      className="mx-auto max-w-xl space-y-6"
      onSubmit={(e) => {
        e.preventDefault()
        onStart()
      }}
    >
      <header className="space-y-1">
        <h1 className="text-2xl font-bold">{t('sentences.title')}</h1>
        <p className="text-slate-600">{t('sentences.subtitle')}</p>
      </header>

      <fieldset className="space-y-1">
        <legend className="mb-1 font-semibold">{t('game.levels')}</legend>
        {NOUN_LEVELS.map((level: Level) => (
          <label key={level} className="flex items-baseline gap-2">
            <input
              type="checkbox"
              checked={settings.levels.includes(level)}
              onChange={() =>
                onChange({
                  ...settings,
                  levels: settings.levels.includes(level)
                    ? settings.levels.filter((l) => l !== level)
                    : [...settings.levels, level],
                })
              }
            />
            {t(`game.level.${level}`)}
          </label>
        ))}
      </fieldset>

      <fieldset>
        <legend className="mb-2 font-semibold">{t('sentences.options.cases')}</legend>
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
          {CASES.map((c) => (
            <label key={c} className={choice}>
              <input
                type="checkbox"
                checked={settings.cases.includes(c)}
                disabled={c === 'nominative' && singular}
                onChange={() => onChange({ ...settings, cases: toggleCase(settings.cases, c) })}
              />
              {t(`cases.${c}`)}
            </label>
          ))}
        </div>
        {singular && settings.cases.includes('nominative') && (
          <p className="mt-1 text-xs text-slate-500">
            {t('sentences.options.nominativePluralOnly')}
          </p>
        )}
      </fieldset>

      <fieldset>
        <legend className="mb-2 font-semibold">{t('sentences.options.number')}</legend>
        <div className="grid grid-cols-3 gap-2">
          {NUMBER_CHOICES.map((n) => (
            <label key={n} className={choice}>
              <input
                type="radio"
                name="number"
                checked={settings.number === n}
                onChange={() => onChange({ ...settings, number: n })}
              />
              {t(`sentences.options.numbers.${n}`)}
            </label>
          ))}
        </div>
      </fieldset>

      <fieldset>
        <legend className="mb-2 font-semibold">{t('sentences.options.roundLength')}</legend>
        <div className="grid grid-cols-3 gap-2">
          {ROUND_LENGTHS.map((n) => (
            <label key={n} className={choice}>
              <input
                type="radio"
                name="roundLength"
                checked={settings.roundLength === n}
                onChange={() => onChange({ ...settings, roundLength: n })}
              />
              {t('sentences.options.sentences', { count: n })}
            </label>
          ))}
        </div>
      </fieldset>

      <fieldset className="space-y-2">
        <legend className="mb-1 font-semibold">{t('sentences.options.help')}</legend>
        <label className="flex items-baseline gap-2">
          <input
            type="checkbox"
            checked={settings.endingsOnly}
            onChange={(e) => onChange({ ...settings, endingsOnly: e.target.checked })}
          />
          <span>
            {t('sentences.options.endingsOnly')}
            <span className="block text-sm text-slate-600">
              {t('sentences.options.endingsOnlyHint')}
            </span>
          </span>
        </label>
        <label className="flex items-baseline gap-2">
          <input
            type="checkbox"
            checked={settings.hideTranslation}
            onChange={(e) => onChange({ ...settings, hideTranslation: e.target.checked })}
          />
          <span>
            {t('sentences.options.hideTranslation')}
            <span className="block text-sm text-slate-600">
              {t('sentences.options.hideTranslationHint')}
            </span>
          </span>
        </label>
      </fieldset>

      {(error || noCase) && (
        <p role="alert" className="text-sm text-red-600">
          {noCase
            ? t('sentences.options.noCases')
            : t(error === 'empty' ? 'sentences.empty' : 'game.failed')}
        </p>
      )}
      <Button
        type="submit"
        disabled={settings.levels.length === 0 || noCase || starting}
        className="w-full"
      >
        {starting ? t('common.loading') : t('game.start')}
      </Button>
    </form>
  )
}
