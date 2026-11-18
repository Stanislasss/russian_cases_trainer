import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Modal } from '../ui/Modal'
import { CaseRules } from './CaseRules'
import type { Case } from './paradigms'
import { useGrammarTexts } from './texts'

interface Props {
  gameCase: Case
  /** The word's table row (a paradigm id) to open the rules at. */
  anchor?: string
  className?: string
}

/** "All the rules for this case →": the case's rules over the game, so the game
 *  carries on where it was once they're closed. */
export function CaseRulesLink({ gameCase, anchor, className = '' }: Props) {
  const { t } = useTranslation()
  const g = useGrammarTexts()
  const [open, setOpen] = useState(false)

  return (
    <>
      <button
        type="button"
        onClick={() => setOpen(true)}
        className={`inline-block text-sky-700 underline ${className}`}
      >
        {g.readMore} →
      </button>
      {open && (
        <Modal title={t(`cases.${gameCase}`)} onClose={() => setOpen(false)}>
          <CaseRules gameCase={gameCase} anchor={anchor} />
        </Modal>
      )}
    </>
  )
}
