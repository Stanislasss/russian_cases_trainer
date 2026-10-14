import { fireEvent, render, screen } from '@testing-library/react'
import { expect, test } from 'vitest'
import i18n from '.'
import { LanguageSwitcher } from './LanguageSwitcher'

test('switching language updates i18next', async () => {
  await i18n.changeLanguage('en')
  render(<LanguageSwitcher />)

  fireEvent.change(screen.getByLabelText('Language'), { target: { value: 'ru' } })

  expect(i18n.resolvedLanguage).toBe('ru')
  expect(await screen.findByLabelText('Язык')).toBeInTheDocument()
})
