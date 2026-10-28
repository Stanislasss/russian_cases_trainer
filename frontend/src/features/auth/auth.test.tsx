import { fireEvent, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest'
import i18n from '../../shared/i18n'
import { json, mockApi, problem, renderAt, requestBodies } from '../../test/app'
import { safeNext } from './safeNext'

const anna = { id: '1', email: 'anna@example.com', display_name: 'Anna', ui_locale: 'en' }

function fill(label: string, value: string) {
  fireEvent.change(screen.getByLabelText(label), { target: { value } })
}

beforeEach(() => i18n.changeLanguage('en'))
afterEach(() => vi.unstubAllGlobals())

describe('login', () => {
  test('validates before calling the API', async () => {
    const fetchMock = mockApi({})
    renderAt('/login')

    fill('Email', 'not-an-email')
    fireEvent.click(screen.getByRole('button', { name: 'Log in' }))

    expect(await screen.findByText('Enter a valid email address.')).toBeInTheDocument()
    expect(screen.getByText('Required.')).toBeInTheDocument()
    expect(await requestBodies(fetchMock, '/api/auth/login')).toEqual([])
  })

  test('shows wrong credentials', async () => {
    mockApi({ 'POST /api/auth/login': () => problem(401, 'invalid_credentials') })
    renderAt('/login')

    fill('Email', 'anna@example.com')
    fill('Password', 'wrong password')
    fireEvent.click(screen.getByRole('button', { name: 'Log in' }))

    expect(await screen.findByRole('alert')).toHaveTextContent('Wrong email or password.')
  })

  test('returns to the protected page after logging in', async () => {
    mockApi({ 'POST /api/auth/login': () => json(200, anna) })
    const router = renderAt('/account')

    // Logged out: the guard sends us to the login page, remembering where we were.
    await screen.findByRole('heading', { name: 'Log in' })
    expect(router.state.location.search).toBe('?next=%2Faccount')

    fill('Email', 'anna@example.com')
    fill('Password', 'correct horse')
    fireEvent.click(screen.getByRole('button', { name: 'Log in' }))

    expect(await screen.findByRole('heading', { name: 'Your account' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Anna' })).toBeInTheDocument()
  })
})

describe('register', () => {
  test('puts server errors on the matching field', async () => {
    mockApi({ 'POST /api/auth/register': () => problem(409, 'email_taken') })
    renderAt('/register')

    fill('Name', 'Anna')
    fill('Email', 'anna@example.com')
    fill('Password', 'correct horse')
    fireEvent.click(screen.getByRole('button', { name: 'Create account' }))

    expect(await screen.findByText('This email is already registered.')).toBeInTheDocument()
    expect(screen.getByLabelText('Email')).toHaveAttribute('aria-invalid', 'true')
  })

  test('sends the current UI language', async () => {
    await i18n.changeLanguage('fr')
    const fetchMock = mockApi({ 'POST /api/auth/register': () => json(201, anna) })
    renderAt('/register')

    fill('Nom', 'Anna')
    fill('E-mail', 'anna@example.com')
    fill('Mot de passe', 'correct horse')
    fireEvent.click(screen.getByRole('button', { name: 'Créer le compte' }))

    await screen.findByRole('link', { name: 'Anna' })
    const [request] = await requestBodies(fetchMock, '/api/auth/register')
    expect(request).toMatchObject({ ui_locale: 'fr', email: 'anna@example.com' })
  })
})

test('safeNext only allows in-app paths', () => {
  expect(safeNext('/account')).toBe('/account')
  expect(safeNext('//evil.example')).toBe('/')
  expect(safeNext('https://evil.example')).toBe('/')
  expect(safeNext(null)).toBe('/')
})

test('logged-in players find the word settings on their account page', async () => {
  mockApi({ 'GET /api/me': () => json(200, anna) })
  renderAt('/account')
  const setting = await screen.findByRole('checkbox', { name: /Show words I know less often/ })
  await waitFor(() => expect(setting).toBeChecked())
})
