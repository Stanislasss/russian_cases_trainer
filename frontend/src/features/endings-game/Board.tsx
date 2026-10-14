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
import { Button } from '../../shared/ui/Button'
import { endingLabel } from './format'
import { ARCADE_LIVES, type Playing, PRACTICE_ROUND, type Word } from './gameReducer'

interface Props {
  state: Playing
  onAnswer: (wordId: number, ending: string) => void
  onQuit: () => void
}

const zoneId = (ending: string) => `zone:${ending}`
const wordId = (id: number) => `word:${id}`

/**
 * The playing field. Answer by dragging a word onto an ending, by tapping a word
 * then an ending, or with the number keys (tapping/keys without a selected word
 * answer the oldest word on the board).
 */
export function Board({ state, onAnswer, onQuit }: Props) {
  const { t } = useTranslation()
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
      const index = Number(e.key) - 1
      if (Number.isInteger(index) && index >= 0 && index < state.zones.length && target) {
        onAnswer(target.id, state.zones[index])
        setSelected(null)
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [state.zones, target, onAnswer])

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

  return (
    <DndContext
      sensors={sensors}
      onDragStart={onDragStart}
      onDragEnd={onDragEnd}
      onDragCancel={() => setDragging(null)}
    >
      <section className="mx-auto flex max-w-3xl flex-col gap-4">
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
          <Button variant="ghost" onClick={onQuit}>
            {t('game.quit')}
          </Button>
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
                  chosen: endingLabel(last.chosen),
                  form: last.item.stressed_form,
                })}
              </span>
            ))}
        </p>

        <div className="grid grid-cols-3 gap-2 sm:grid-cols-4">
          {state.zones.map((ending, i) => (
            <Zone key={ending} ending={ending} shortcut={i + 1} onClick={() => answer(ending)} />
          ))}
        </div>
        <p className="text-center text-xs text-slate-500">{t('game.howTo')}</p>
      </section>

      <DragOverlay dropAnimation={null}>
        {dragging && <CardFace word={dragging} selected={false} />}
      </DragOverlay>
    </DndContext>
  )
}

function Zone({
  ending,
  shortcut,
  onClick,
}: {
  ending: string
  shortcut: number
  onClick: () => void
}) {
  const { t } = useTranslation()
  const { setNodeRef, isOver } = useDroppable({ id: zoneId(ending) })
  return (
    <button
      ref={setNodeRef}
      type="button"
      onClick={onClick}
      aria-label={ending === '' ? t('game.zeroEnding') : endingLabel(ending)}
      className={`relative rounded-lg border-2 py-5 text-xl font-semibold transition-colors ${
        isOver ? 'border-sky-500 bg-sky-50' : 'border-slate-300 bg-white hover:border-slate-400'
      }`}
    >
      {endingLabel(ending)}
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
}

function WordCard({ word, arcade, selected, hidden, onSelect }: CardProps) {
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
      <CardFace word={word} selected={selected} />
    </div>
  )
}

function CardFace({ word, selected }: { word: Word; selected: boolean }) {
  const { t } = useTranslation()
  return (
    <div
      className={`cursor-grab rounded-lg border-2 bg-white px-4 py-2 text-center shadow-sm ${
        selected ? 'border-sky-500 ring-2 ring-sky-200' : 'border-slate-300'
      }`}
    >
      <div className="text-2xl font-semibold">{word.item.stressed_lemma}</div>
      <div className="text-xs text-slate-500">
        <span className="mr-1 rounded bg-slate-100 px-1 font-medium text-slate-700">
          {t(`game.numbers.${word.item.number}`)}
        </span>
        {word.item.translation}
      </div>
    </div>
  )
}
