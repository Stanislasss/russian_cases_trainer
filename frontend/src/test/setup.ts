import '@testing-library/jest-dom/vitest'
import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'
import '../shared/i18n'

// Testing Library only auto-unmounts when Vitest globals are on; they aren't here.
afterEach(cleanup)
