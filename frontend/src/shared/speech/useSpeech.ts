import { useCallback, useState } from 'react'

const MUTED_KEY = 'speech.muted'

/** Audio of a Russian word, synthesized and cached by the backend. */
export function speechUrl(text: string) {
  return `${window.location.origin}/api/speech?text=${encodeURIComponent(text)}`
}

// One sound at a time across the app, plus what's waiting to play after it.
let playing: HTMLAudioElement | null = null
let queue: string[] = []

function playNext() {
  const text = queue.shift()
  if (text === undefined) {
    playing = null
    return
  }
  const audio = new Audio(speechUrl(text))
  playing = audio
  // Move on once per sound, and never on behalf of a sound that was stopped.
  let finished = false
  const next = () => {
    if (finished || playing !== audio) return
    finished = true
    playNext()
  }
  audio.addEventListener('ended', next)
  audio.addEventListener('error', next)
  // Browsers refuse audio before the first user gesture; sound is a bonus, not essential.
  audio.play()?.catch(next)
}

function stop() {
  queue = []
  playing?.pause()
  playing = null
}

export type Speech = ReturnType<typeof useSpeech>

/**
 * Pronounces Russian words. `speak` interrupts whatever is playing; `enqueue`
 * waits for it to finish. Muting is remembered across visits.
 */
export function useSpeech() {
  const [muted, setMuted] = useState(() => localStorage.getItem(MUTED_KEY) === 'true')

  const speak = useCallback(
    (text: string) => {
      if (muted) return
      stop()
      queue = [text]
      playNext()
    },
    [muted],
  )

  const enqueue = useCallback(
    (text: string) => {
      if (muted) return
      queue.push(text)
      // A paused sound never started (blocked, or failed to load): don't wait on it.
      if (!playing || playing.paused) playNext()
    },
    [muted],
  )

  /** Asks the backend to synthesize now, so a later `speak` plays at once. */
  const prefetch = useCallback(
    (text: string) => {
      if (!muted) fetch(speechUrl(text)).catch(() => {})
    },
    [muted],
  )

  const toggleMuted = useCallback(() => {
    setMuted((wasMuted) => {
      localStorage.setItem(MUTED_KEY, String(!wasMuted))
      if (!wasMuted) stop()
      return !wasMuted
    })
  }, [])

  return { speak, enqueue, prefetch, muted, toggleMuted }
}
