import { useCallback, useState } from 'react'
import { speechAvailable, useConfig } from '../config/config'

const MUTED_KEY = 'speech.muted'

/** Audio of a Russian word, synthesized and cached by the backend. */
export function speechUrl(text: string) {
  return `${window.location.origin}/api/speech?text=${encodeURIComponent(text)}`
}

/** How a request to speak ended: heard to the end, or not heard at all. */
export type Outcome = 'played' | 'skipped'

interface Entry {
  text: string
  done: (outcome: Outcome) => void
}

// One sound at a time across the app, plus what's waiting to play after it.
let playing: { audio: HTMLAudioElement; entry: Entry } | null = null
let queue: Entry[] = []

function playNext() {
  const entry = queue.shift()
  if (!entry) {
    playing = null
    return
  }
  const audio = new Audio(speechUrl(entry.text))
  const current = { audio, entry }
  playing = current
  // Settle once per sound, and never on behalf of a sound that was stopped.
  let finished = false
  const finish = (outcome: Outcome) => {
    if (finished || playing !== current) return
    finished = true
    entry.done(outcome)
    playNext()
  }
  audio.addEventListener('ended', () => finish('played'))
  audio.addEventListener('error', () => finish('skipped'))
  // Browsers refuse audio before the first user gesture; sound is a bonus, not essential.
  audio.play()?.catch(() => finish('skipped'))
}

/** Stops the current sound and drops the queue; everything pending counts as skipped. */
function stop() {
  const pending = [...(playing ? [playing.entry] : []), ...queue]
  queue = []
  playing?.audio.pause()
  playing = null
  pending.forEach((entry) => entry.done('skipped'))
}

export type Speech = ReturnType<typeof useSpeech>

/**
 * Pronounces Russian words. `speak` interrupts whatever is playing; `enqueue`
 * waits for it to finish. Both resolve once the word has been heard, or right
 * away with 'skipped' when speech is off or when the sound can't play.
 *
 * Speech is `available` when the server can pronounce words (the desktop app
 * needs its speech pack for that), and `on` when it's also not muted. Muting is
 * remembered across visits. `listen` is `speak` for 🔊 buttons, absent when
 * speech is off.
 */
export function useSpeech() {
  const [muted, setMuted] = useState(() => localStorage.getItem(MUTED_KEY) === 'true')
  const available = speechAvailable(useConfig().data)
  const on = available && !muted

  const speak = useCallback(
    (text: string): Promise<Outcome> => {
      if (!on) return Promise.resolve('skipped')
      stop()
      return new Promise((done) => {
        queue = [{ text, done }]
        playNext()
      })
    },
    [on],
  )

  const enqueue = useCallback(
    (text: string): Promise<Outcome> => {
      if (!on) return Promise.resolve('skipped')
      return new Promise((done) => {
        queue.push({ text, done })
        // A paused sound never started (blocked, or failed to load): don't wait on it.
        if (playing?.audio.paused) {
          const stale = playing
          playing = null
          stale.entry.done('skipped')
        }
        if (!playing) playNext()
      })
    },
    [on],
  )

  /** Asks the backend to synthesize now, so a later `speak` plays at once. */
  const prefetch = useCallback(
    (text: string) => {
      if (on) fetch(speechUrl(text)).catch(() => {})
    },
    [on],
  )

  const toggleMuted = useCallback(() => {
    setMuted((wasMuted) => {
      localStorage.setItem(MUTED_KEY, String(!wasMuted))
      if (!wasMuted) stop()
      return !wasMuted
    })
  }, [])

  return {
    speak,
    enqueue,
    prefetch,
    muted,
    toggleMuted,
    available,
    on,
    listen: on ? speak : undefined,
  }
}
