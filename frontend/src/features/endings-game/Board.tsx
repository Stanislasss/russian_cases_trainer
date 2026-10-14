import {
  DndContext,
  type DragEndEvent,
  DragOverlay,
  type DragStartEvent,
  KeyboardSensor,
  PointerSensor,
  useDraggable,
  useDroppable,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { Speech } from '../../shared/speech/useSpeech'
import { Button } from '../../shared/ui/Button'
import { answerLabel } from '../../shared/grammar/format'
import { RuleCard } from '../../shared/grammar/RuleCard'
import { useGrammarTexts } from '../../shared/grammar/texts'
import { translationFor } from './format'
import { ARCADE_LIVES, type Playing, PRACTICE_ROUND, type Word } from './gameReducer'

interface Props {
  state: Playing
  onAnswer: (wordId: number, ending: string) => void
  onQuit: () => void
  /** Practice: move on after reading why an answer was wrong. */
  onContinue: () => void
  speech: Speech
}

const zoneId = (ending: string) => `zone:${ending}`
const wordId = (id: number) => `word:${id}`

/**
 * The playing field. Answer by dragging a word onto an ending, by tapping a word
 * then an ending, or with the number keys (tapping/keys without a selected word
 * answer the oldest word on the board).
 */
export function Board({ state, onAnswer, onQuit, onContinue, speech }: Props) {
  const { t } = useTranslation()
  const g = useGrammarTexts()
  const [selected, setSelected] = useState<number | null>(null)
  const [dragging, setDragging] = useState<Word | null>(null)
  const sensors = useSensors(
    // A small distance keeps taps as clicks instead of tiny drags.
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor),
  )

  const target = state.words.find((w) => w.id === selected) ?? state.words[0]
  const answer = (ending: string) => {
    if (!target) return
    onAnswer(target.id, ending)
    setSelected(null)
  }

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (state.paused) {
        if (e.key === 'Enter') onContinue()
        return
      }
      const index = Number(e.key) - 1
      if (Number.isInteger(index) && index >= 0 && index < state.zones.length && target) {
        onAnswer(target.id, state.zones[index])
        setSelected(null)
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [state.zones, state.paused, target, onAnswer, onContinue])

  const onDragStart = (e: DragStartEvent) =>
    setDragging(state.words.find((w) => wordId(w.id) === e.active.id) ?? null)
  const onDragEnd = (e: DragEndEvent) => {
    setDragging(null)
    const word = state.words.find((w) => wordId(w.id) === e.active.id)
    const over = e.over?.id
    if (word && typeof over === 'string' && over.startsWith('zone:')) {
      onAnswer(word.id, over.slice('zone:'.length))
      setSelected(null)
    }
  }

  const last = state.results.at(-1)
  const mistake = state.paused && last && !last.correct && last.chosen !== null ? last : null

  return (
    <DndContext
      sensors={sensors}
      onDragStart={onDragStart}
      onDragEnd={onDragEnd}
      onDragCancel={() => setDragging(null)}
    >
      <div className="mx-auto grid max-w-6xl gap-6 lg:grid-cols-[minmax(0,1fr)_22rem]">
        <section className="flex flex-col gap-4">
          <p className="text-center text-sm text-slate-500">
            {t('game.case')}:{' '}
            <strong className="text-lg text-sky-800">{t(`cases.${state.gameCase}`)}</strong>
          </p>
          <div className="flex items-center justify-between text-sm">
            <span className="font-medium">
              {state.mode === 'practice'
                ? t('game.progress', { done: state.results.length, total: PRACTICE_ROUND })
                : `${t('game.score')}: ${state.score}`}
            </span>
            {state.mode === 'arcade' && (
              <span aria-label={t('game.lives', { count: state.lives })}>
                {'♥'.repeat(state.lives)}
                <span className="text-slate-300">{'♥'.repeat(ARCADE_LIVES - state.lives)}</span>
                {state.streak >= 2 && (
                  <span className="ml-3 text-amber-600">
                    {t('game.streak', { count: state.streak })}
                  </span>
                )}
              </span>
            )}
            <span>
              <Button
                variant="ghost"
                onClick={speech.toggleMuted}
                aria-pressed={speech.muted}
                aria-label={t(speech.muted ? 'game.unmute' : 'game.mute')}
                title={t(speech.muted ? 'game.unmute' : 'game.mute')}
              >
                {speech.muted ? '🔇' : '🔊'}
              </Button>
              <Button variant="ghost" onClick={onQuit}>
                {t('game.quit')}
              </Button>
            </span>
          </div>

          <div
            className={`relative rounded-lg border border-slate-200 bg-white ${
              state.mode === 'arcade'
                ? 'h-[45vh] overflow-hidden'
                : 'flex min-h-40 items-center justify-center'
            }`}
          >
            {state.words.map((word) => (
              <WordCard
                key={word.id}
                word={word}
                arcade={state.mode === 'arcade'}
                selected={word.id === selected}
                hidden={dragging?.id === word.id}
                onSelect={() => setSelected(word.id === selected ? null : word.id)}
                onListen={speech.muted ? undefined : () => speech.speak(word.item.stressed_lemma)}
              />
            ))}
          </div>

          <p aria-live="polite" className="min-h-6 text-center text-sm">
            {last &&
              (last.correct ? (
                <span className="text-green-700">✓ {last.item.stressed_form}</span>
              ) : last.chosen === null ? (
                <span className="text-red-700">
                  {t('game.feedback.missed', { form: last.item.stressed_form })}
                </span>
              ) : (
                <span className="text-red-700">
                  {t('game.feedback.wrong', {
                    chosen: answerLabel(last.chosen, state.answerKind),
                    form: last.item.stressed_form,
                  })}
                </span>
              ))}
          </p>

          <div className="grid grid-cols-3 gap-2 sm:grid-cols-4">
            {state.zones.map((zone, i) => (
              <Zone
                key={zone}
                zone={zone}
                label={answerLabel(zone, state.answerKind)}
                shortcut={i + 1}
                onClick={() => answer(zone)}
                disabled={state.paused}
              />
            ))}
          </div>
          <p className="text-center text-xs text-slate-500">{t('game.howTo')}</p>
        </section>

        {/* Beside the board on wide screens, below it on phones. */}
        {mistake && mistake.chosen !== null && (
          <div className="space-y-3">
            <RuleCard
              item={mistake.item}
              chosen={mistake.chosen}
              gameCase={state.gameCase}
              answerKind={state.answerKind}
              speak={speech.muted ? undefined : speech.speak}
            />
            {/* Focused: reading, then Enter, is the only thing to do now. */}
            <Button onClick={onContinue} autoFocus className="w-full">
              {g.explanation.continue}
            </Button>
          </div>
        )}
      </div>

      <DragOverlay dropAnimation={null}>
        {dragging && <CardFace word={dragging} selected={false} />}
      </DragOverlay>
    </DndContext>
  )
}

function Zone({
  zone,
  label,
  shortcut,
  onClick,
  disabled,
}: {
  zone: string
  label: string
  disabled?: boolean
  shortcut: number
  onClick: () => void
}) {
  const { t } = useTranslation()
  const { setNodeRef, isOver } = useDroppable({ id: zoneId(zone) })
  return (
    <button
      ref={setNodeRef}
      type="button"
      onClick={onClick}
      disabled={disabled}
      aria-label={zone === '' ? t('game.zeroEnding') : label}
      className={`relative rounded-lg border-2 py-5 text-xl font-semibold transition-colors disabled:opacity-40 ${
        isOver ? 'border-sky-500 bg-sky-50' : 'border-slate-300 bg-white hover:border-slate-400'
      }`}
    >
      {label}
      {shortcut <= 9 && (
        <span className="absolute top-1 left-2 text-xs font-normal text-slate-400">{shortcut}</span>
      )}
    </button>
  )
}

interface CardProps {
  word: Word
  arcade: boolean
  selected: boolean
  hidden: boolean
  onSelect: () => void
  onListen?: () => void
}

function WordCard({ word, arcade, selected, hidden, onSelect, onListen }: CardProps) {
  const { attributes, listeners, setNodeRef } = useDraggable({ id: wordId(word.id) })
  // Spread words across the width; deterministic so they don't jump on re-render.
  const left = 5 + ((word.id * 37) % 60)
  return (
    <div
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      onClick={onSelect}
      className={`touch-none select-none ${arcade ? 'absolute animate-fall' : ''} ${hidden ? 'opacity-30' : ''}`}
      style={arcade ? { left: `${left}%`, animationDuration: `${word.duration}ms` } : undefined}
    >
      <CardFace word={word} selected={selected} onListen={onListen} />
    </div>
  )
}

interface CardFaceProps {
  word: Word
  selected: boolean
  /** Shows a 🔊 button; absent when muted and on the copy being dragged. */
  onListen?: () => void
}

function CardFace({ word, selected, onListen }: CardFaceProps) {
  const { t, i18n } = useTranslation()
  return (
    <div
      className={`max-w-xs cursor-grab rounded-lg border-2 bg-white px-4 py-2 text-center shadow-sm ${
        selected ? 'border-sky-500 ring-2 ring-sky-200' : 'border-slate-300'
      }`}
    >
      <div className="flex items-center justify-center gap-1">
        <span className="text-2xl font-semibold">{word.item.stressed_lemma}</span>
        {onListen && (
          <button
            type="button"
            aria-label={t('game.listen')}
            title={t('game.listen')}
            className="rounded px-1 text-lg hover:bg-slate-100"
            // Keep the card's drag, selection and keyboard handling out of it.
            onPointerDown={(e) => e.stopPropagation()}
            onKeyDown={(e) => e.stopPropagation()}
            onClick={(e) => {
              e.stopPropagation()
              onListen()
            }}
          >
            🔊
          </button>
        )}
      </div>
      <div className="flex flex-wrap items-center justify-center gap-1 text-xs text-slate-500">
        {/* Which form is asked: gender and animacy matter for adjective-like words. */}
        {[word.item.gender, word.item.animacy, word.item.number].map(
          (badge) =>
            badge && (
              <span key={badge} className="rounded bg-slate-100 px-1 font-medium text-slate-700">
                {t(`game.badges.${badge}`)}
              </span>
            ),
        )}
        {/* Picked at render time, so switching language mid-game updates it. */}
        <span className="line-clamp-2">{translationFor(word.item, i18n.resolvedLanguage)}</span>
      </div>
    </div>
  )
}
