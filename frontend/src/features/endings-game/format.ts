/** Endings as the player sees them: "-ой", or "∅" for the zero ending. */
export function endingLabel(ending: string) {
  return ending === '' ? '∅' : `-${ending}`
}
