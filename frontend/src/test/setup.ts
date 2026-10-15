import '@testing-library/jest-dom/vitest'
import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'
import '../shared/i18n'

// Testing Library only auto-unmounts when Vitest globals are on; they aren't here.
afterEach(cleanup)

// jsdom has no modal dialogs: open and close them without the top layer.
HTMLDialogElement.prototype.showModal ??= function (this: HTMLDialogElement) {
  this.open = true
}
HTMLDialogElement.prototype.close ??= function (this: HTMLDialogElement) {
  if (!this.open) return
  this.open = false
  // Browsers queue the close event rather than firing it at once.
  setTimeout(() => this.dispatchEvent(new Event('close')))
}
