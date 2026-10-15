import { act, render, screen } from '@testing-library/react'
import { StrictMode } from 'react'
import { expect, test, vi } from 'vitest'
import { Modal } from './Modal'

test('stays open when StrictMode mounts it twice', async () => {
  const onClose = vi.fn()
  render(
    <StrictMode>
      <Modal title="Rules" onClose={onClose}>
        text
      </Modal>
    </StrictMode>,
  )
  await act(() => new Promise((resolve) => setTimeout(resolve, 10)))
  expect(onClose).not.toHaveBeenCalled()
  expect(screen.getByRole('dialog', { name: 'Rules' })).toHaveAttribute('open')
})
